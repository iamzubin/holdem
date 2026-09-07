import { useCallback, useEffect, useRef } from 'react';
import { FilePreview } from '@/types';
import { setPendingFiles, prepareDragImage, triggerNativeDrag } from '@/lib/fileUtils';

const DRAG_DELAY_MS = 150;

/**
 * Canonical native-drag sequence shared by PopupWindow (per-file rows) and
 * StackedIcons (whole stack). `beginDrag` captures the drag image from
 * `element` then starts the OS drag after a short delay; `cancelDrag`
 * aborts a pending drag on mouse-up / unmount.
 */
export const useNativeDrag = () => {
  const isMouseDownRef = useRef(false);
  const dragTimeoutRef = useRef<number | null>(null);

  const cancelDrag = useCallback(() => {
    isMouseDownRef.current = false;
    if (dragTimeoutRef.current !== null) {
      clearTimeout(dragTimeoutRef.current);
      dragTimeoutRef.current = null;
    }
  }, []);

  const beginDrag = useCallback(
    (filesToDrag: FilePreview[], element: HTMLElement | null) => {
      isMouseDownRef.current = true;
      setPendingFiles(filesToDrag);

      const schedule = () => {
        if (!isMouseDownRef.current) return;
        dragTimeoutRef.current = window.setTimeout(() => {
          if (isMouseDownRef.current) triggerNativeDrag();
        }, DRAG_DELAY_MS);
      };

      if (element) {
        prepareDragImage(element).then(() => {
          if (isMouseDownRef.current) schedule();
        });
      } else {
        schedule();
      }
    },
    [],
  );

  useEffect(() => cancelDrag, [cancelDrag]);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent, filesToDrag: FilePreview[], element: HTMLElement | null) => {
      if (e.button !== 0) return;
      e.stopPropagation();
      beginDrag(filesToDrag, element);
    },
    [beginDrag],
  );

  return { beginDrag, cancelDrag, handleMouseDown, handleMouseUp: cancelDrag };
};
