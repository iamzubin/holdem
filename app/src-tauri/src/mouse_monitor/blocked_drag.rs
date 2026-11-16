//! Drags Windows never lets reach Holdem. A drag between processes at
//! different integrity levels (an app running as administrator and one that
//! isn't) is blocked by Windows: the drop target is never called, the user
//! just sees the "no" cursor. This notices that case and tells them why.

use crate::drop_report::{DropIssue, DropReport};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use tauri::AppHandle;
use tracing::info;
use windows::core::w;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, POINT};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::WindowsAndMessaging::{
    GetAncestor, GetCursorPos, GetForegroundWindow, GetWindowThreadProcessId, PostMessageW,
    RegisterWindowMessageW, WindowFromPoint, GA_ROOT,
};

/// Set by the drop target when any drag enters a Holdem window.
pub(crate) static DRAG_ENTERED: AtomicBool = AtomicBool::new(false);

/// Follows one press of the drag button.
#[derive(Default)]
pub(super) struct BlockedDragWatch {
    was_down: bool,
    /// The window in front when the press began: the drag's source. None
    /// for presses that began on Holdem itself (its own drags reach its drop
    /// target normally).
    source: Option<isize>,
    /// At the last check, a drag was over a Holdem window and its drop
    /// target had not been called.
    blocked_over_us: bool,
}

impl BlockedDragWatch {
    /// Call on every check. `dragging` tells whether a real drag is under
    /// way; it is only asked while the pointer is over Holdem.
    pub(super) fn tick(&mut self, down: bool, dragging: impl Fn() -> bool, app: &AppHandle) {
        if down && !self.was_down {
            DRAG_ENTERED.store(false, Ordering::SeqCst);
            let source = unsafe { GetForegroundWindow() };
            self.source = (!source.is_invalid() && !pointer_over_holdem() && !is_ours(source))
                .then_some(source.0 as isize);
            self.blocked_over_us = false;
        } else if down {
            // Where the pointer is when the button comes up is what counts:
            // a drag that only passes over the shelf was not meant for it.
            self.blocked_over_us = self.source.is_some()
                && !DRAG_ENTERED.load(Ordering::SeqCst)
                && pointer_over_holdem()
                && dragging();
        } else if self.was_down {
            // Released over Holdem with still no DragEnter: Windows kept the
            // drop from reaching it.
            if self.blocked_over_us && !DRAG_ENTERED.load(Ordering::SeqCst) {
                if let Some(issue) = self.source.and_then(|s| blocked_reason(HWND(s as _))) {
                    info!("A drag over Holdem was blocked by Windows ({:?})", issue);
                    DropReport::single(issue).send(app);
                }
            }
            self.source = None;
            self.blocked_over_us = false;
        }
        self.was_down = down;
    }
}

/// True when the pointer is over one of Holdem's own windows.
fn pointer_over_holdem() -> bool {
    let mut point = POINT::default();
    if unsafe { GetCursorPos(&mut point) }.is_err() {
        return false;
    }
    is_ours(unsafe { GetAncestor(WindowFromPoint(point), GA_ROOT) })
}

/// True when `window` belongs to Holdem.
fn is_ours(window: HWND) -> bool {
    if window.is_invalid() {
        return false;
    }
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(window, Some(&mut pid)) };
    pid == std::process::id()
}

/// Why Windows would block a drag from `source` to Holdem, if it would.
fn blocked_reason(source: HWND) -> Option<DropIssue> {
    if holdem_is_elevated() {
        // Every drag from a normal app to an elevated Holdem is blocked.
        return Some(DropIssue::HoldemElevated);
    }
    // Windows refuses messages from a lower to a higher integrity level
    // (UIPI) exactly as it refuses the drag. A private registered message is
    // ignored by every window, so the probe has no effect when it goes
    // through.
    let probe = unsafe { RegisterWindowMessageW(w!("HoldemIntegrityProbe")) };
    if probe == 0 {
        return None;
    }
    let refused = unsafe { PostMessageW(Some(source), probe, Default::default(), Default::default()) }
        .is_err_and(|e| e.code() == windows::Win32::Foundation::E_ACCESSDENIED);
    refused.then_some(DropIssue::ElevatedSource)
}

/// True when Holdem itself runs as administrator. Checked once.
pub(crate) fn holdem_is_elevated() -> bool {
    static ELEVATED: OnceLock<bool> = OnceLock::new();
    *ELEVATED.get_or_init(|| unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        )
        .is_ok();
        let _ = CloseHandle(token);
        ok && elevation.TokenIsElevated != 0
    })
}
