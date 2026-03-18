# Hold'em - Cross-Platform File Drag App

## Summary

Hold'em is a desktop app that appears when users shake their mouse while dragging files, allowing quick file transfers. Currently supports Windows and macOS.

---

## Pending Tasks

### 1. Fix macOS Crash on Drag & Image Previews 🔄 IN PROGRESS

**Issue 1:** App crashes when starting to drag on macOS with null pointer dereference
**Location:** Unknown - appears after file drop event
**Error:** `thread 'main' (5243827) panicked at ... null pointer dereference occurred`

**Issue 2:** Image previews not working on macOS
- Shows generic file icon with "PNG" text instead of actual image content
- Using `file_icon_provider` crate which returns QuickLook preworks but isn't rendering correctly
- Error: `osascript failed: 140:151: execution error: The variable NSWorkspace is not defined.`

**Current implementation in `src/utils/thumbnails.rs`:**
- macOS: Uses `file_icon_provider::get_file_icon(path, 256)` for images/videos
- Windows: Uses `auto-thumbnail` crate for actual thumbnails
- Falls back to `file_icon_provider` for non-image/video files

**Root cause analysis:**
- `file_icon_provider` on macOS uses QuickLook but returns the file type icon instead of content preview for some files
- The crash during drag appears to be unrelated to icon fetching - happens after icon lookup fails

**Required fixes:**
1. Investigate and fix the null pointer dereference crash during drag
2. Get actual image content thumbnails working on macOS

**Approach for macOS thumbnails:**
- Use NSWorkspace.icon(forFile:) to get Finder icons (file type icons)
- Use qlmanage or a different method for actual image content
- Research: file_icon_provider may need specific configuration

---

## Dependencies

```toml
device_query = "1.0"
file_icon_provider = "1.0"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.61.3", features = ["Win32_UI_Input_KeyboardAndMouse"] }
auto-thumbnail = "0.1"
```
