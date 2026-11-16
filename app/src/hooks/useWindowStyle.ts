import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';

/** Sent by Settings when Mica is turned on or off; every window re-applies. */
export const WINDOW_STYLE_EVENT = 'window-style-changed';

/**
 * Windows 11 look for the current window: rounded corners and, if turned on
 * in Settings, the Mica backdrop. When Mica is active the page gets
 * `<html data-mica>` and its background turns translucent (global.css);
 * otherwise it stays solid, so a failure never leaves a see-through window.
 */
export function useWindowStyle(): void {
  useEffect(() => {
    const root = document.documentElement;
    let mica = true;
    const apply = () => {
      invoke<boolean>('apply_window_style', { mica, dark: root.classList.contains('dark') })
        .then((on) => {
          if (on) root.dataset.mica = '';
          else delete root.dataset.mica;
        })
        .catch(() => delete root.dataset.mica);
    };

    invoke<{ mica?: boolean }>('get_config')
      .then((config) => {
        mica = config.mica !== false;
      })
      .catch(() => {})
      .finally(apply);

    // Light and dark Mica differ, so follow theme changes.
    const observer = new MutationObserver(apply);
    observer.observe(root, { attributes: true, attributeFilter: ['class'] });
    const unlisten = listen<boolean>(WINDOW_STYLE_EVENT, (event) => {
      mica = event.payload;
      apply();
    });
    return () => {
      observer.disconnect();
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, []);
}
