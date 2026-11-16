//! Tells a shelf item whose file was deleted apart from one whose drive or
//! network share is just not reachable right now (unplugged USB stick,
//! sleeping NAS, VPN down, mapped drive not connected yet at sign-in).
//! Deleted items leave the shelf; unreachable ones stay, shown as offline,
//! and come back by themselves once the location is reachable again.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    /// The file is there.
    Present,
    /// Its drive or share answers, but the file is gone.
    Gone,
    /// Its drive or share is offline, or did not answer in time.
    Unreachable,
}

/// The volume a path lives on: `C:\`, or `\\server\share\` for UNC paths.
fn volume_root(path: &Path) -> PathBuf {
    path.components()
        .take_while(|c| matches!(c, Component::Prefix(_) | Component::RootDir))
        .collect()
}

/// Checks every path, grouped by volume. Each volume is probed on its own
/// thread: an unplugged drive fails at once, but a dead network share can
/// block a file-system call for half a minute, so whatever has not answered
/// within `timeout` counts as unreachable. A probe that is still stuck keeps
/// running in the background and is ignored.
pub fn check(paths: &[PathBuf], timeout: Duration) -> Vec<Presence> {
    let mut by_volume: HashMap<PathBuf, Vec<usize>> = HashMap::new();
    for (index, path) in paths.iter().enumerate() {
        by_volume.entry(volume_root(path)).or_default().push(index);
    }

    let mut result = vec![Presence::Unreachable; paths.len()];
    let (tx, rx) = mpsc::channel::<Vec<(usize, Presence)>>();
    let groups = by_volume.len();
    for (root, indices) in by_volume {
        let tx = tx.clone();
        let group: Vec<(usize, PathBuf)> =
            indices.into_iter().map(|i| (i, paths[i].clone())).collect();
        std::thread::spawn(move || {
            let answer = if root.as_os_str().is_empty() || root.metadata().is_ok() {
                group
                    .into_iter()
                    .map(|(i, path)| {
                        let found = if path.exists() { Presence::Present } else { Presence::Gone };
                        (i, found)
                    })
                    .collect()
            } else {
                group.into_iter().map(|(i, _)| (i, Presence::Unreachable)).collect()
            };
            let _ = tx.send(answer);
        });
    }
    drop(tx);

    let deadline = Instant::now() + timeout;
    for _ in 0..groups {
        let left = deadline.saturating_duration_since(Instant::now());
        match rx.recv_timeout(left) {
            Ok(answers) => {
                for (i, presence) in answers {
                    result[i] = presence;
                }
            }
            Err(_) => break,
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_roots() {
        assert_eq!(volume_root(Path::new(r"C:\a\b.txt")), PathBuf::from(r"C:\"));
        assert_eq!(
            volume_root(Path::new(r"\\nas\photos\2026\a.jpg")),
            PathBuf::from(r"\\nas\photos\")
        );
    }

    #[test]
    fn present_gone_and_unreachable() {
        let dir = std::env::temp_dir().join(format!("holdem_presence_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("dir");
        let here = dir.join("here.txt");
        std::fs::write(&here, b"x").expect("write");
        let gone = dir.join("gone.txt");
        // A drive letter that is almost never mapped.
        let offline = PathBuf::from(r"B:\holdem\offline.txt");

        let found = check(&[here, gone, offline.clone()], Duration::from_secs(5));
        assert_eq!(found[0], Presence::Present);
        assert_eq!(found[1], Presence::Gone);
        if !volume_root(&offline).exists() {
            assert_eq!(found[2], Presence::Unreachable);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn keeps_input_order_across_volumes() {
        let temp = std::env::temp_dir();
        let paths = vec![PathBuf::from(r"B:\x"), temp.clone(), PathBuf::from(r"B:\y"), temp];
        let found = check(&paths, Duration::from_secs(5));
        assert_eq!(found[1], Presence::Present);
        assert_eq!(found[3], Presence::Present);
    }
}
