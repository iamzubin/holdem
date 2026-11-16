//! Windows 11 look for Holdem's frameless windows: rounded corners, and a
//! see-through blur of whatever is behind the window.
//!
//! Windows 11's own backdrops (Mica, and Acrylic through
//! DWMWA_SYSTEMBACKDROP_TYPE) are drawn only while the window is active and
//! turn solid otherwise, and the shelf is almost never the active window.
//! So the blur comes from the window's accent policy
//! (`SetWindowCompositionAttribute`, ACCENT_ENABLE_ACRYLICBLURBEHIND), which
//! Windows applies whether or not the window is active. It is undocumented
//! but has been stable since Windows 10 1803. On Windows 10, or with the
//! setting off, windows stay solid. (The setting is still stored as `mica`
//! in config.json.)

use tauri::WebviewWindow;
use tracing::warn;

/// First Windows 11 build; Mica and rounded corners need it.
const WINDOWS_11_BUILD: u32 = 22000;

fn windows_build() -> u32 {
    use windows::core::w;
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};
    let mut buf = [0u16; 16];
    let mut size = std::mem::size_of_val(&buf) as u32;
    let read = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            w!(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion"),
            w!("CurrentBuildNumber"),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    if read.is_err() {
        return 0;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len]).trim().parse().unwrap_or(0)
}

fn round_corners(window: &WebviewWindow) {
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    };
    let Ok(hwnd) = window.hwnd() else { return };
    let preference = DWMWCP_ROUND;
    let set = unsafe {
        DwmSetWindowAttribute(
            windows::Win32::Foundation::HWND(hwnd.0 as _),
            DWMWA_WINDOW_CORNER_PREFERENCE,
            std::ptr::from_ref(&preference).cast(),
            std::mem::size_of_val(&preference) as u32,
        )
    };
    if let Err(e) = set {
        warn!("Could not round the window corners: {}", e);
    }
}

/// ACCENT_POLICY, as `SetWindowCompositionAttribute` takes it.
#[repr(C)]
struct AccentPolicy {
    state: u32,
    flags: u32,
    /// Tint, as 0xAABBGGRR.
    gradient_color: u32,
    animation_id: u32,
}

/// WINDOWCOMPOSITIONATTRIBDATA.
#[repr(C)]
struct CompositionData {
    attribute: u32,
    data: *mut std::ffi::c_void,
    size: usize,
}

const WCA_ACCENT_POLICY: u32 = 19;
const ACCENT_DISABLED: u32 = 0;
const ACCENT_ENABLE_ACRYLICBLURBEHIND: u32 = 4;

/// Turns the blur behind the window on (with a light or dark tint) or off.
/// Returns false when Windows doesn't offer it.
fn set_blur(window: &WebviewWindow, on: bool, dark: bool) -> bool {
    use windows::core::{s, w, BOOL};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
    type SetWindowCompositionAttribute =
        unsafe extern "system" fn(HWND, *mut CompositionData) -> BOOL;

    let Ok(hwnd) = window.hwnd() else { return false };
    let Some(set) = (unsafe {
        GetModuleHandleW(w!("user32.dll"))
            .ok()
            .and_then(|user32| GetProcAddress(user32, s!("SetWindowCompositionAttribute")))
    }) else {
        return false;
    };
    // SAFETY: user32's SetWindowCompositionAttribute has this signature.
    let set: SetWindowCompositionAttribute = unsafe { std::mem::transmute(set) };
    // A light tint keeps a little contrast for the page's own translucent
    // background on top; 0x01 alpha would be no tint at all.
    let tint = if dark { 0x40_20_20_20 } else { 0x40_f3_f3_f3 };
    let mut policy = AccentPolicy {
        state: if on { ACCENT_ENABLE_ACRYLICBLURBEHIND } else { ACCENT_DISABLED },
        flags: 0,
        gradient_color: tint,
        animation_id: 0,
    };
    let mut data = CompositionData {
        attribute: WCA_ACCENT_POLICY,
        data: std::ptr::from_mut(&mut policy).cast(),
        size: std::mem::size_of::<AccentPolicy>(),
    };
    unsafe { set(HWND(hwnd.0 as _), &mut data) }.as_bool()
}

/// Applies the style to the calling window. Returns whether the see-through
/// backdrop is on, so the page knows to make its background translucent.
#[tauri::command]
pub fn apply_window_style(window: WebviewWindow, mica: bool, dark: bool) -> bool {
    if windows_build() < WINDOWS_11_BUILD {
        return false;
    }
    round_corners(&window);
    // No system backdrop (an older Holdem set Mica): it would draw a solid
    // colour over the blur whenever the window is inactive.
    let _ = window.set_effects(None);
    if !mica {
        set_blur(&window, false, dark);
        return false;
    }
    if set_blur(&window, true, dark) {
        true
    } else {
        warn!("Could not turn on the see-through backdrop");
        false
    }
}
