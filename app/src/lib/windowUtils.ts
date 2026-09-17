import { getCurrentWindow } from '@tauri-apps/api/window';

/**
 * Hide the main window (tray-resident; closing would quit the app).
 * Secondary windows (popup/settings/consent/updater) should use
 * `closeCurrentWindow()` which actually destroys the webview.
 */
export const hideMainWindow = (): Promise<void> =>
  getCurrentWindow().hide().catch((error) => {
    console.error('Failed to hide window:', error);
  });

/**
 * Canonical closer for secondary windows. Replaces the backend
 * `close_popup_window` / `close_settings_window` round-trips and the
 * one-off `getCurrentWindow().close()` in Updater — all three do the
 * same `.close()`, so call this directly.
 */
export const closeCurrentWindow = (): Promise<void> =>
  getCurrentWindow().close().catch((error) => {
    console.error('Failed to close window:', error);
  });