import React, { useMemo, useRef } from 'react';
import { FilePreview } from '@/types';
import { FileThumb } from './FileThumb';
import { useNativeDrag } from '@/hooks/useNativeDrag';

interface StackedIconsProps {
  files: FilePreview[];
}

export const StackedIcons: React.FC<StackedIconsProps> = ({ files }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const { beginDrag } = useNativeDrag();

  const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;
    e.stopPropagation();
    // Items on an offline drive or share can't be dragged anywhere.
    const reachable = files.filter((file) => file.available);
    if (reachable.length > 0) beginDrag(reachable, containerRef.current, e);
  };

  const stackedIcons = useMemo(() => {
    return files.slice(-5).reverse().map((file, index) => {
      const rotation = -index * 10;
      const zIndex = files.length - index;

      return (
        <div
          key={file.id}
          className="absolute inset-0 flex items-center pointer-events-none"
          style={{
            transform: `rotate(${rotation}deg) translate(${-index}px, ${-index}px)`,
            zIndex,
          }}
        >
          <FileThumb file={file} />
        </div>
      );
    });
  }, [files]);

  return (
    <div
      ref={containerRef}
      // touch-none: a touch drag is Holdem's drag, not a pan the browser
      // would take over (and cancel with pointercancel).
      className="relative w-full h-full cursor-grab touch-none active:cursor-grabbing"
      onPointerDown={handlePointerDown}
      // Holdem starts the drag itself; the page's own image drag would compete.
      onDragStart={(e) => e.preventDefault()}
    >
      {stackedIcons}
    </div>
  );
};
