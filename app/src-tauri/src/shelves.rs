//! Named shelves. Every shelf item belongs to one shelf (`FileMetadata::shelf`);
//! the shelf window and the item list show the active one, and new drops go
//! there. Shelves are saved with their items by `shelf_store`.

use crate::file::FileMetadata;
use crate::presence::{self, Presence};
use crate::FileList;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shelf {
    pub id: u64,
    /// Empty for the first shelf: the page shows its translated default name.
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShelfSet {
    pub shelves: Vec<Shelf>,
    pub active: u64,
    /// Recently closed shelves, newest first, kept so they can be reopened.
    #[serde(default)]
    pub closed: Vec<ClosedShelf>,
}

impl Default for ShelfSet {
    fn default() -> Self {
        Self {
            shelves: vec![Shelf { id: FIRST_SHELF, name: String::new() }],
            active: FIRST_SHELF,
            closed: Vec::new(),
        }
    }
}

/// A closed shelf: its name, place and items, for "Reopen closed shelf".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosedShelf {
    pub shelf: Shelf,
    /// Its position among the tabs.
    pub index: usize,
    pub items: Vec<ClosedItem>,
    /// When it was closed, in seconds since 1970.
    pub closed_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClosedItem {
    pub path: PathBuf,
    #[serde(default)]
    pub pinned: bool,
}

/// Closed shelves kept, at most. Far above normal use: they expire by age,
/// and their files stay safe from cleanup until then.
const MAX_CLOSED: usize = 50;
/// Closed shelves are kept this long. Their items' files stay safe from the
/// drop-folder cleanup until then.
const KEEP_CLOSED: Duration = Duration::from_secs(7 * 24 * 60 * 60);
/// How long reopening waits for drives and shares to answer.
const REOPEN_TIMEOUT: Duration = Duration::from_secs(2);

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

pub const FIRST_SHELF: u64 = 1;
const MAX_NAME_CHARS: usize = 40;

pub type Shelves = Arc<Mutex<ShelfSet>>;

/// The active shelf, readable without a lock: new items are created with it.
static ACTIVE: AtomicU64 = AtomicU64::new(FIRST_SHELF);

pub fn active_id() -> u64 {
    ACTIVE.load(Ordering::SeqCst)
}

impl ShelfSet {
    /// Repairs a loaded set: at least one shelf, unique ids, a valid active
    /// shelf. Then publishes the active id.
    pub fn normalized(mut self) -> Self {
        let mut seen = std::collections::HashSet::new();
        self.shelves.retain(|s| seen.insert(s.id));
        if self.shelves.is_empty() {
            self = Self::default();
        }
        if !self.contains(self.active) {
            self.active = self.shelves[0].id;
        }
        let open: std::collections::HashSet<u64> = self.shelves.iter().map(|s| s.id).collect();
        self.closed.retain(|c| !open.contains(&c.shelf.id) && seen.insert(c.shelf.id));
        self.prune_closed(now_secs());
        ACTIVE.store(self.active, Ordering::SeqCst);
        self
    }

    /// Forgets closed shelves past their keeping time or beyond the limit.
    fn prune_closed(&mut self, now: u64) {
        let mut expired: Vec<ClosedShelf> = Vec::new();
        self.closed.retain(|c| {
            let keep = now.saturating_sub(c.closed_at) < KEEP_CLOSED.as_secs();
            if !keep {
                expired.push(c.clone());
            }
            keep
        });
        if self.closed.len() > MAX_CLOSED {
            expired.extend(self.closed.drain(MAX_CLOSED..));
        }
        // Their items leave Holdem now: keep their files a while longer.
        crate::file_drop::mark_left_shelf(
            expired.iter().flat_map(|c| c.items.iter().map(|i| i.path.as_path())),
        );
    }

    /// Every path a closed shelf still holds: the drop-folder cleanup must
    /// leave them alone.
    pub fn closed_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.closed.iter().flat_map(|c| c.items.iter().map(|i| &i.path))
    }

    pub fn contains(&self, id: u64) -> bool {
        self.shelves.iter().any(|s| s.id == id)
    }

    fn set_active(&mut self, id: u64) {
        self.active = id;
        ACTIVE.store(id, Ordering::SeqCst);
    }

    /// Never the id of an open or a closed shelf, so a reopened shelf can't
    /// collide with one made since.
    fn next_id(&self) -> u64 {
        self.shelves
            .iter()
            .map(|s| s.id)
            .chain(self.closed.iter().map(|c| c.shelf.id))
            .max()
            .unwrap_or(0)
            + 1
    }
}

fn clean_name(name: &str) -> String {
    name.trim().chars().filter(|c| !c.is_control()).take(MAX_NAME_CHARS).collect()
}

fn lock<'a>(shelves: &'a State<'_, Shelves>) -> Result<std::sync::MutexGuard<'a, ShelfSet>, String> {
    shelves.lock().map_err(|_| "Failed to acquire lock".to_string())
}

fn changed(app: &AppHandle) -> Result<(), String> {
    app.emit("files_updated", ()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_shelves(shelves: State<'_, Shelves>) -> Result<ShelfSet, String> {
    Ok(lock(&shelves)?.clone())
}

/// Adds a shelf and makes it the active one. Returns its id.
#[tauri::command]
pub fn create_shelf(app: AppHandle, shelves: State<'_, Shelves>, name: String) -> Result<u64, String> {
    let id = {
        let mut set = lock(&shelves)?;
        let id = set.next_id();
        set.shelves.push(Shelf { id, name: clean_name(&name) });
        set.set_active(id);
        id
    };
    changed(&app)?;
    Ok(id)
}

#[tauri::command]
pub fn rename_shelf(
    app: AppHandle,
    shelves: State<'_, Shelves>,
    id: u64,
    name: String,
) -> Result<(), String> {
    {
        let mut set = lock(&shelves)?;
        let shelf = set
            .shelves
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or("No such shelf")?;
        shelf.name = clean_name(&name);
    }
    changed(&app)
}

#[tauri::command]
pub fn set_active_shelf(app: AppHandle, shelves: State<'_, Shelves>, id: u64) -> Result<(), String> {
    {
        let mut set = lock(&shelves)?;
        if !set.contains(id) {
            return Err("No such shelf".to_string());
        }
        if set.active == id {
            return Ok(());
        }
        set.set_active(id);
    }
    changed(&app)
}

/// Closes a shelf and takes its items off. It is kept, with its items, for
/// [`reopen_shelf`]. The last shelf can't be closed. Returns the item count.
#[tauri::command]
pub fn delete_shelf(
    app: AppHandle,
    shelves: State<'_, Shelves>,
    file_list: State<'_, FileList>,
    id: u64,
) -> Result<usize, String> {
    let (shelf, index) = {
        let mut set = lock(&shelves)?;
        if set.shelves.len() <= 1 {
            return Err("The last shelf can't be deleted".to_string());
        }
        let index = set.shelves.iter().position(|s| s.id == id).ok_or("No such shelf")?;
        let shelf = set.shelves.remove(index);
        if set.active == id {
            let next = set.shelves[index.min(set.shelves.len() - 1)].id;
            set.set_active(next);
        }
        (shelf, index)
    };
    // Items leave the list and enter `closed` under both locks (list first,
    // as everywhere), so the drop-folder cleanup never sees them in neither.
    let count = {
        let mut list = file_list.lock().map_err(|_| "Failed to acquire lock".to_string())?;
        let mut set = lock(&shelves)?;
        let (removed, kept): (Vec<FileMetadata>, Vec<FileMetadata>) =
            list.drain(..).partition(|f| f.shelf == id);
        *list = kept;
        let count = removed.len();
        let items = removed
            .into_iter()
            .map(|f| ClosedItem { path: f.path, pinned: f.pinned })
            .collect();
        set.closed.insert(0, ClosedShelf { shelf, index, items, closed_at: now_secs() });
        set.prune_closed(now_secs());
        count
    };
    changed(&app)?;
    Ok(count)
}

/// Reopens a closed shelf (`id`, or the most recently closed one) in its old
/// place, with its items, and makes it the active shelf. Items whose file
/// was deleted meanwhile stay off, with the usual message. Returns its id.
#[tauri::command]
pub async fn reopen_shelf(app: AppHandle, id: Option<u64>) -> Result<u64, String> {
    // A copy: the entry stays in `closed` (its id reserved, its files safe
    // from cleanup) until the shelf is back in the list below.
    let closed = {
        let shelves = app.state::<Shelves>();
        let set = shelves.lock().map_err(|_| "Failed to acquire lock".to_string())?;
        match id {
            Some(id) => set.closed.iter().find(|c| c.shelf.id == id),
            None => set.closed.first(),
        }
        .cloned()
        .ok_or("No closed shelf to reopen")?
    };
    let shelf_id = closed.shelf.id;
    let items = closed.items;
    // Bounded: a share that doesn't answer must not hold the shelf up.
    let found = tauri::async_runtime::spawn_blocking({
        let paths: Vec<PathBuf> = items.iter().map(|i| i.path.clone()).collect();
        move || presence::check(&paths, REOPEN_TIMEOUT)
    })
    .await
    .map_err(|e| e.to_string())?;
    let mut missing = 0;
    let files: Vec<FileMetadata> = items
        .into_iter()
        .zip(found)
        .filter_map(|(item, presence)| {
            let file = match presence {
                Presence::Present => FileMetadata::from_path(item.path.clone())
                    .unwrap_or_else(|| FileMetadata::unavailable(item.path, item.pinned)),
                Presence::Unreachable => FileMetadata::unavailable(item.path, item.pinned),
                Presence::Gone => {
                    missing += 1;
                    return None;
                }
            };
            Some(FileMetadata { pinned: item.pinned, shelf: shelf_id, ..file })
        })
        .collect();
    {
        // Both locks (list first, as everywhere): the items move from
        // `closed` back to the list in one step.
        let file_list = app.state::<FileList>();
        let mut list = file_list.lock().map_err(|_| "Failed to acquire lock".to_string())?;
        let shelves = app.state::<Shelves>();
        let mut set = shelves.lock().map_err(|_| "Failed to acquire lock".to_string())?;
        // Reopened meanwhile (a second click): nothing more to do.
        let Some(position) = set.closed.iter().position(|c| c.shelf.id == shelf_id) else {
            return Ok(shelf_id);
        };
        set.closed.remove(position);
        let index = closed.index.min(set.shelves.len());
        set.shelves.insert(index, closed.shelf);
        set.set_active(shelf_id);
        for file in files {
            if !list.iter().any(|f| f.path == file.path && f.shelf == file.shelf) {
                list.push(file);
            }
        }
    }
    crate::commands::file_ops::notify_missing(&app, missing);
    changed(&app)?;
    Ok(shelf_id)
}

/// Moves items to another shelf. Items already there (same file) are
/// dropped instead of duplicated. Returns how many moved.
#[tauri::command]
pub fn move_to_shelf(
    app: AppHandle,
    shelves: State<'_, Shelves>,
    file_list: State<'_, FileList>,
    ids: Vec<u64>,
    shelf: u64,
) -> Result<usize, String> {
    if !lock(&shelves)?.contains(shelf) {
        return Err("No such shelf".to_string());
    }
    let moved = {
        let mut list = file_list.lock().map_err(|_| "Failed to acquire lock".to_string())?;
        move_items(&mut list, &ids, shelf)
    };
    if moved > 0 {
        changed(&app)?;
    }
    Ok(moved)
}

fn move_items(list: &mut Vec<FileMetadata>, ids: &[u64], shelf: u64) -> usize {
    let already: std::collections::HashSet<std::path::PathBuf> =
        list.iter().filter(|f| f.shelf == shelf).map(|f| f.path.clone()).collect();
    let mut moved = 0;
    list.retain_mut(|f| {
        if !ids.contains(&f.id) || f.shelf == shelf {
            return true;
        }
        moved += 1;
        if already.contains(&f.path) {
            return false;
        }
        f.shelf = shelf;
        true
    });
    moved
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u64, path: &str, shelf: u64) -> FileMetadata {
        let mut f = FileMetadata::unavailable(path.into(), false);
        f.id = id;
        f.shelf = shelf;
        f
    }

    #[test]
    fn normalizing_repairs_empty_and_bad_active() {
        let set = ShelfSet { shelves: vec![], active: 9, closed: vec![] }.normalized();
        assert_eq!(set, ShelfSet::default());
        let set = ShelfSet {
            shelves: vec![Shelf { id: 4, name: "a".into() }, Shelf { id: 4, name: "b".into() }],
            active: 9,
            closed: vec![],
        }
        .normalized();
        assert_eq!(set.shelves.len(), 1);
        assert_eq!(set.active, 4);
        ShelfSet::default().normalized();
    }

    #[test]
    fn moving_skips_duplicates_on_the_target() {
        let mut list = vec![item(1, r"C:\a", 1), item(2, r"C:\b", 1), item(3, r"C:\a", 2)];
        assert_eq!(move_items(&mut list, &[1, 2], 2), 2);
        // C:\a was already on shelf 2: the copy from shelf 1 is gone.
        assert_eq!(list.len(), 2);
        assert!(list.iter().all(|f| f.shelf == 2));
    }

    fn closed(id: u64, closed_at: u64) -> ClosedShelf {
        ClosedShelf {
            shelf: Shelf { id, name: String::new() },
            index: 0,
            items: vec![ClosedItem { path: format!(r"C:\\{id}").into(), pinned: false }],
            closed_at,
        }
    }

    #[test]
    fn closed_shelves_expire_and_keep_their_ids_reserved() {
        let now = now_secs();
        let set = ShelfSet {
            closed: vec![closed(5, now), closed(6, now - KEEP_CLOSED.as_secs() - 1)],
            ..Default::default()
        };
        let set = set.normalized();
        assert_eq!(set.closed.len(), 1);
        // A new shelf never takes a closed shelf's id.
        assert_eq!(set.next_id(), 6);
        assert_eq!(set.closed_paths().count(), 1);
    }

    #[test]
    fn a_closed_shelf_with_an_open_shelfs_id_is_dropped() {
        let set = ShelfSet { closed: vec![closed(FIRST_SHELF, now_secs())], ..Default::default() };
        assert!(set.normalized().closed.is_empty());
    }

    #[test]
    fn names_are_trimmed_and_capped() {
        assert_eq!(clean_name("  Work\n "), "Work");
        assert_eq!(clean_name(&"x".repeat(100)).len(), MAX_NAME_CHARS);
    }
}
