import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { FilePreview } from "../types.ts";

/**
 * Draw the drag image: the element's thumbnails stacked like StackedIcons,
 * with a badge showing `fileCount` when more than one file is dragged.
 * Returns null when the element has no loaded thumbnails (the backend then
 * uses its default image).
 */
async function captureElementAsImage(
  element: HTMLElement,
  fileCount: number,
): Promise<string | null> {
  try {
    const rect = element.getBoundingClientRect();
    const targetSize = 256;

    const canvas = document.createElement('canvas');
    canvas.width = targetSize;
    canvas.height = targetSize;
    const ctx = canvas.getContext('2d');
    if (!ctx) return null;

    // Scale the element to fill 75% of the canvas, leaving padding around it.
    const maxDim = Math.max(rect.width, rect.height);
    const drawScale = (targetSize * 0.75) / maxDim;

    // Draw all img children (thumbnails/previews) onto the canvas
    const images = element.querySelectorAll('img');
    const imageElements = Array.from(images);

    // If we have actual preview images, draw them stacked like the StackedIcons component
    if (imageElements.length > 0) {
      const maxToShow = Math.min(imageElements.length, 5);
      
      for (let i = maxToShow - 1; i >= 0; i--) {
        const img = imageElements[i] as HTMLImageElement;
        if (!img.complete || !img.naturalWidth) continue;

        const rotation = -i * 10 * (Math.PI / 180);
        const translateX = -i;
        const translateY = -i;

        ctx.save();
        ctx.translate(targetSize / 2 + translateX * drawScale, targetSize / 2 + translateY * drawScale);
        ctx.rotate(rotation);

        const drawSize = Math.min(rect.width, rect.height) * drawScale;
        ctx.drawImage(img, -drawSize / 2, -drawSize / 2, drawSize, drawSize);
        ctx.restore();
      }

      // The stack shows at most 5 thumbnails, and a popup row shows one, so
      // the badge counts the files being dragged, not the images drawn.
      if (fileCount > 1) {
        drawCountBadge(ctx, targetSize, fileCount);
      }

      return canvas.toDataURL('image/png');
    }

    return null;
  } catch (e) {
    console.error('Failed to capture element as image:', e);
    return null;
  }
}

function drawCountBadge(ctx: CanvasRenderingContext2D, canvasSize: number, count: number) {
  const radius = 22;
  const cx = canvasSize - radius - 4;
  const cy = canvasSize - radius - 4;

  // Badge circle
  ctx.beginPath();
  ctx.arc(cx, cy, radius, 0, Math.PI * 2);
  ctx.fillStyle = '#3b82f6'; // Blue
  ctx.fill();
  ctx.strokeStyle = 'white';
  ctx.lineWidth = 3;
  ctx.stroke();

  // Badge count text
  const label = count > 99 ? '99+' : String(count);
  ctx.fillStyle = 'white';
  ctx.font = `bold ${count > 9 ? 16 : 20}px -apple-system, sans-serif`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(label, cx, cy);
}

// Store drag image data to be used synchronously
let pendingDragImage: string | null = null;
// Store pending files for drag
let pendingFiles: FilePreview[] = [];

export const prepareDragImage = async (element: HTMLElement, fileCount: number): Promise<void> => {
  pendingDragImage = await captureElementAsImage(element, fileCount);
};

export const setPendingFiles = (files: FilePreview[]): void => {
  pendingFiles = files;
};

/**
 * Trigger native drag for the pending files.
 * Call this synchronously from mousedown to avoid browser drag conflicts.
 */
export const triggerNativeDrag = (): void => {
  if (pendingFiles.length === 0) {
    return;
  }

  const dragImage = pendingDragImage;
  
  // Clear after use
  const filesToDrag = [...pendingFiles];
  pendingFiles = [];
  pendingDragImage = null;

  // Focus window first to enable drag from unfocused window
  const window = getCurrentWindow();
  window.setFocus().catch((error) => {
    console.error('Failed to focus window before starting drag:', error);
  });

  // Start native drag after a small delay to ensure window is focused
  setTimeout(async () => {
    if (
      filesToDrag.length === 1 &&
      // Dropped text snippets are saved as note_<time>_<id>.txt.
      filesToDrag[0].name.startsWith('note_') &&
      filesToDrag[0].name.endsWith('.txt')
    ) {
      try {
        const { readTextFile } = await import('@tauri-apps/plugin-fs');
        const text = await readTextFile(filesToDrag[0].path);
        await invoke('start_text_drag', {
          text,
          dragImage,
        });
        return;
      } catch (error) {
        console.error('Failed to read and drag text snippet:', error);
      }
    }

    invoke('start_multi_drag', {
      filePaths: filesToDrag.map((file) => file.path),
      dragImage,
    }).catch((error) => {
      console.error('Failed to start native drag:', error);
    });
  }, 50);
};
