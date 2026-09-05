import React, { useMemo, useRef } from 'react';
import { FilePreview } from '@/types';
import { FileThumb } from './FileThumb';
import { useNativeDrag } from '@/hooks/useNativeDrag';

interface StackedIconsProps {
  files: FilePreview[];
}

export const StackedIcons: React.FC<StackedIconsProps> = ({ files }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const { beginDrag, handleMouseUp } = useNativeDrag();

  const handleMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;
    e.stopPropagation();
    beginDrag(files, containerRef.current);
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
          <FileThumb file={file} className="w-full h-full object-cover" />
        </div>
      );
    });
  }, [files]);

  return (
    <div
      ref={containerRef}
      className="relative w-full h-full cursor-grab active:cursor-grabbing"
      onMouseDown={handleMouseDown}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
    >
      {stackedIcons}
    </div>
  );
};
