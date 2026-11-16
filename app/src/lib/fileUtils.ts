import { invoke } from "@tauri-apps/api/core";
import { FilePreview } from "@/types";

/**
 * Draw the drag image: the element's thumbnails stacked like StackedIcons,
 * with a badge showing `fileCount` when more than one file is dragged.
 * Returns null when the element has no loaded thumbnails (the backend then
 * uses its default image).
 */
export async function captureElementAsImage(
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

/**
 * Starts the OS drag for `files`, with `dragImage` (a PNG data URL) or the
 * backend's default image. Must run while the button is still held.
 */
export const startNativeDrag = async (
  files: FilePreview[],
  dragImage: string | null,
  /** False for pen and touch, whose contact isn't a held mouse button. */
  mouse = true,
): Promise<void> => {
  // A saved text snippet goes out as text, so it pastes into chats and
  // editors; the backend reads it by shelf id.
  if (files.length === 1 && files[0].kind === 'note') {
    try {
      await invoke('start_note_drag', { id: files[0].id, dragImage, mouse });
      return;
    } catch (error) {
      console.error('Failed to drag text snippet, dragging the file instead:', error);
    }
  }
  try {
    await invoke('start_multi_drag', {
      filePaths: files.map((file) => file.path),
      dragImage,
      mouse,
    });
  } catch (error) {
    console.error('Failed to start native drag:', error);
  }
};
