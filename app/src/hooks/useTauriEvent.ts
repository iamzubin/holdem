import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useEffect, useRef } from 'react';

/**
 * Listens for a backend event while the component is mounted: events sent to
 * every window, and those sent to this window only (not another window's).
 * The handler may change between renders without re-subscribing, and an
 * unmount before `listen` resolves still unsubscribes.
 */
export function useTauriEvent<T>(event: string, handler: (payload: T) => void): void {
  const handlerRef = useRef(handler);
  handlerRef.current = handler;

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    getCurrentWebviewWindow()
      .listen<T>(event, (e) => handlerRef.current(e.payload))
      .then((fn) => {
        if (cancelled) fn();
        else unlisten = fn;
      })
      .catch((error) => console.error(`Failed to listen for ${event}:`, error));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [event]);
}
