import { useCallback, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { FilePreview } from '@/types';
import { captureElementAsImage, startNativeDrag } from '@/lib/fileUtils';

/** Pen and touch wobble more than a mouse: at least this many CSS px. */
const PEN_TOUCH_MIN_PX = 10;

/** Windows' drag distance in physical pixels, as last read. */
let threshold = { x: 4, y: 4 };

/** Reads the user's drag distance; it follows their setting and the DPI. */
const readThreshold = () => {
  invoke<[number, number]>('drag_threshold')
    .then(([x, y]) => {
      threshold = { x, y };
    })
    .catch(() => {});
};

/**
 * Native drag-out shared by PopupWindow (per-item rows) and StackedIcons
 * (the whole stack). As in Explorer, a press becomes a drag only once the
 * pointer moves further than Windows' drag distance (SM_CXDRAG/SM_CYDRAG);
 * a press that ends sooner, however long, stays a click. The drag image is
 * drawn from `element` (its `[data-drag-image]` part, if marked) when the
 * drag starts. Leaving the element doesn't cancel: a quick flick off a small
 * row is a real drag. Esc before that point cancels the press.
 */
export const useNativeDrag = () => {
  const cancelRef = useRef<(() => void) | null>(null);

  const cancelDrag = useCallback(() => {
    cancelRef.current?.();
    cancelRef.current = null;
  }, []);

  // Read once, and again whenever the window comes back to the front: the
  // setting or the monitor (DPI) may have changed meanwhile.
  useEffect(() => {
    readThreshold();
    window.addEventListener('focus', readThreshold);
    return () => window.removeEventListener('focus', readThreshold);
  }, []);

  const beginDrag = useCallback(
    (files: FilePreview[], element: HTMLElement | null, event: React.PointerEvent) => {
      cancelDrag();
      const target = event.currentTarget as HTMLElement;
      const { pointerId, pointerType } = event;
      const start = { x: event.screenX, y: event.screenY };
      // Keep receiving moves when the pointer leaves the element or window.
      try {
        target.setPointerCapture(pointerId);
      } catch {
        // Already released: the press is over.
        return;
      }

      const cleanup = () => {
        target.removeEventListener('pointermove', onMove);
        target.removeEventListener('pointerup', cleanup);
        target.removeEventListener('pointercancel', cleanup);
        target.removeEventListener('lostpointercapture', cleanup);
        window.removeEventListener('keydown', onKey, true);
        if (target.hasPointerCapture(pointerId)) target.releasePointerCapture(pointerId);
        cancelRef.current = null;
      };
      // Captured first, so this Esc only cancels the press and doesn't also
      // clear the selection or close the window.
      const onKey = (e: KeyboardEvent) => {
        if (e.key !== 'Escape') return;
        e.stopPropagation();
        e.preventDefault();
        cleanup();
      };
      const onMove = (e: PointerEvent) => {
        // Screen coordinates are in CSS pixels; Windows' distance is physical.
        const scale = window.devicePixelRatio || 1;
        const min = pointerType === 'mouse' ? 0 : PEN_TOUCH_MIN_PX * scale;
        const dx = Math.abs(e.screenX - start.x) * scale;
        const dy = Math.abs(e.screenY - start.y) * scale;
        if (dx <= Math.max(threshold.x, min) && dy <= Math.max(threshold.y, min)) return;
        cleanup();
        const source = element?.querySelector<HTMLElement>('[data-drag-image]') ?? element;
        const image = source ? captureElementAsImage(source, files.length) : Promise.resolve(null);
        image.then((dragImage) => startNativeDrag(files, dragImage, pointerType === 'mouse'));
      };

      target.addEventListener('pointermove', onMove);
      target.addEventListener('pointerup', cleanup);
      target.addEventListener('pointercancel', cleanup);
      target.addEventListener('lostpointercapture', cleanup);
      window.addEventListener('keydown', onKey, true);
      cancelRef.current = cleanup;
    },
    [cancelDrag],
  );

  useEffect(() => cancelDrag, [cancelDrag]);

  return { beginDrag, cancelDrag };
};
