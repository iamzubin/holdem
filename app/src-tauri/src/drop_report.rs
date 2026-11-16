//! What happened to the items of one drop (or paste), told to the user in a
//! single message: how many were added, and why the others were not.

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tracing::warn;

/// Why something dropped on the shelf was not added as the user expected.
/// The page turns each one into a sentence (`lib/notices.ts`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DropIssue {
    /// Holdem can't keep anything in the dropped data.
    Unsupported,
    /// One item is over the per-item cap (`MAX_ITEM_MB`).
    TooLarge,
    /// The drop as a whole is over the per-drop cap (`MAX_DROP_MB`) or has
    /// more than `MAX_DROP_FILES` items; the rest was left out.
    DropTooLarge,
    /// The other app listed an item but didn't hand over its contents
    /// (typically an attachment that isn't downloaded yet).
    NoContents,
    /// The other app stopped handing over an item part-way.
    ReadFailed,
    /// A web image couldn't be downloaded, so its link was kept instead.
    SavedAsLink,
    /// No space left on the drive Holdem saves to.
    DiskFull,
    /// Windows (often security software) refused the save.
    AccessDenied,
    /// Saving failed for another reason.
    SaveFailed,
    /// A dropped file or folder can't be opened (gone, no access, offline).
    Unreadable,
    /// Already on this shelf; nothing to add. Informational.
    AlreadyOnShelf,
    /// Windows blocks drags from an app running as administrator.
    ElevatedSource,
    /// Holdem itself runs as administrator, so Windows blocks drags from
    /// every normal app.
    HoldemElevated,
    /// Another app has the clipboard open; pasting may work in a moment.
    ClipboardBusy,
}

/// Per-item cap for data that exists only in the drop, in MB.
pub const MAX_ITEM_MB: usize = 64;
/// Per-drop cap for such data, in MB.
pub const MAX_DROP_MB: usize = 256;
/// Most items read from one virtual-file drop.
pub const MAX_DROP_FILES: usize = 64;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct IssueCount {
    reason: DropIssue,
    count: usize,
}

/// Collects the outcome of one drop. Sent once, when the drop is finished.
#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DropReport {
    issues: Vec<IssueCount>,
    /// The item's name, when the report is about exactly one item.
    name: Option<String>,
    /// Items that did get added, so "1 already on the shelf" isn't read as
    /// "nothing was added".
    added: usize,
}

/// The limits, sent with each report so the page never repeats them.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Limits {
    item_mb: usize,
    drop_mb: usize,
    files: usize,
}

impl DropReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn single(issue: DropIssue) -> Self {
        let mut report = Self::new();
        report.add(issue, 1);
        report
    }

    pub fn add(&mut self, issue: DropIssue, count: usize) {
        if count == 0 {
            return;
        }
        match self.issues.iter_mut().find(|i| i.reason == issue) {
            Some(existing) => existing.count += count,
            None => self.issues.push(IssueCount { reason: issue, count }),
        }
    }

    /// Like [`add`](Self::add) for one item; its name is shown when it
    /// turns out to be the only one.
    pub fn add_named(&mut self, issue: DropIssue, name: &str) {
        self.add(issue, 1);
        self.name = Some(name.to_string());
    }

    pub fn set_added(&mut self, added: usize) {
        self.added = added;
    }

    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    /// Shows the message on the shelf and the list, if there is anything to
    /// tell. Analytics get the reasons and counts, never names.
    pub fn send(mut self, app: &AppHandle) {
        if self.is_empty() {
            return;
        }
        let total: usize = self.issues.iter().map(|i| i.count).sum();
        if total != 1 {
            self.name = None;
        }
        let limits = Limits { item_mb: MAX_ITEM_MB, drop_mb: MAX_DROP_MB, files: MAX_DROP_FILES };
        let payload = serde_json::json!({ "kind": "drop", "report": &self, "limits": limits });
        // To every window: the drop may have landed on the shelf or the list.
        if let Err(e) = app.emit("shelf-notice", payload) {
            warn!("Failed to emit shelf-notice: {}", e);
        }
        for issue in &self.issues {
            let reason = serde_json::to_value(issue.reason).unwrap_or_default();
            crate::analytics::track(
                app,
                "drop_issue",
                vec![("reason", reason), ("count", issue.count.into())],
            );
        }
    }
}

/// The issue for a failed save, from the underlying I/O error.
pub fn save_issue(error: &crate::commands::file_ops::SaveError) -> DropIssue {
    use crate::commands::file_ops::SaveError;
    use std::io::ErrorKind;
    // ERROR_HANDLE_DISK_FULL, ERROR_DISK_FULL
    const DISK_FULL: [i32; 2] = [39, 112];
    match error {
        SaveError::Io(io) if io.kind() == ErrorKind::StorageFull => DropIssue::DiskFull,
        SaveError::Io(io) if io.raw_os_error().is_some_and(|c| DISK_FULL.contains(&c)) => {
            DropIssue::DiskFull
        }
        SaveError::Io(io) if io.kind() == ErrorKind::PermissionDenied => DropIssue::AccessDenied,
        SaveError::NotALink(_) => DropIssue::Unsupported,
        SaveError::Io(_) => DropIssue::SaveFailed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_merge_by_reason_and_names_only_single_items() {
        let mut report = DropReport::new();
        report.add_named(DropIssue::TooLarge, "video.mp4");
        report.add(DropIssue::TooLarge, 2);
        report.add(DropIssue::DiskFull, 0);
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues[0].count, 3);
    }

    #[test]
    fn disk_full_is_recognised_from_the_os_error() {
        use crate::commands::file_ops::SaveError;
        let full = SaveError::Io(std::io::Error::from_raw_os_error(112));
        assert_eq!(save_issue(&full), DropIssue::DiskFull);
        let denied = SaveError::Io(std::io::Error::from_raw_os_error(5));
        assert_eq!(save_issue(&denied), DropIssue::AccessDenied);
        assert_eq!(save_issue(&SaveError::NotALink("x".into())), DropIssue::Unsupported);
        let other = SaveError::Io(std::io::Error::other("x"));
        assert_eq!(save_issue(&other), DropIssue::SaveFailed);
    }
}
