import { invoke } from "@tauri-apps/api/core";
import { FilePreview } from "../types.ts";

/**
 * Capture an HTML element as a PNG data URL using canvas.
 * Returns null if capture fails.
 */
async function captureElementAsImage(element: HTMLElement): Promise<string | null> {
  try {
    // Use html2canvas-style approach with native canvas
    const rect = element.getBoundingClientRect();
    const scale = window.devicePixelRatio || 2; // Retina/HiDPI support
    const targetSize = 256; // 256px as user requested (hi-quality for Mac Retina)
    
    const canvas = document.createElement('canvas');
    canvas.width = targetSize;
    canvas.height = targetSize;
    const ctx = canvas.getContext('2d');
    if (!ctx) return null;

    // Scale and center the element content
    const maxDim = Math.max(rect.width, rect.height);
    const drawScale = (targetSize * 0.75) / maxDim; // Leave 12.5% padding on each side

    const offsetX = (targetSize - rect.width * drawScale) / 2;
    const offsetY = (targetSize - rect.height * drawScale) / 2;

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

      // Draw file count badge if multiple files
      if (imageElements.length > 1) {
        drawCountBadge(ctx, targetSize, imageElements.length);
      }

      return canvas.toDataURL('image/png');
    }

    // Fallback: draw a generic file icon if no images
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

export const handleDragStart = async (e: React.DragEvent<HTMLDivElement>, file: FilePreview) => {
  e.preventDefault();
  e.stopPropagation();

  // No text/plain data — the native drag from the backend provides file URLs to the pasteboard
  e.dataTransfer.effectAllowed = 'copyMove';

  try {
    await invoke('start_multi_drag', { filePaths: [file.path], dragImage: null });
  } catch (error) {
    console.error('Error starting drag:', error);
  }
};

export const handleMultiFileDragStart = async (
  e: React.DragEvent<HTMLDivElement>,
  files: FilePreview[],
  dragSourceElement?: HTMLElement
) => {
  e.preventDefault();
  e.stopPropagation();

  // No text/plain — native drag handles file URLs properly
  e.dataTransfer.effectAllowed = 'copyMove';

  // Capture the drag source element as a PNG for the cursor image
  let dragImage: string | null = null;
  if (dragSourceElement) {
    dragImage = await captureElementAsImage(dragSourceElement);
  }

  try {
    await invoke('start_multi_drag', {
      filePaths: files.map(file => file.path),
      dragImage,
    });
  } catch (error) {
    console.error('Error starting multi-file drag:', error);
  }
};