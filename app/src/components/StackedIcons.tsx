import React, { useMemo, useRef, useCallback } from 'react';
import { DynamicFileIcon } from './FileIcon';
import { FilePreview } from '@/types';
import { setPendingFiles, prepareDragImage, triggerNativeDrag } from '@/lib/fileUtils';

interface StackedIconsProps {
  files: FilePreview[];
}

export const StackedIcons: React.FC<StackedIconsProps> = ({ files }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const dragTimeoutRef = useRef<number | null>(null);

  const stackedIcons = useMemo(() => {
    return files.slice(-5).reverse().map((file, index) => {
      const rotation = - index * 10; // Adjust rotation angle as needed
      const translateX = -index;
      const translateY = -index;
      const zIndex = files.length - index;
    
      return (
        <div
          key={index}
          className="absolute inset-0 flex items-center pointer-events-none"
          style={{
            transform: `rotate(${rotation}deg) translate(${translateX}px, ${translateY}px)`,
            zIndex,
          }}
        >
          {file.preview ? (
            <img 
              src={file.preview} 
              alt={file.name} 
              className="w-full h-full object-cover"
              loading="lazy"
            />
          ) : (
            <>
              <DynamicFileIcon file={file} />
            </>
          )}
        </div>
      );
    });
  }, [files]);

  const handleMouseDown = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    // Only trigger on left mouse button
    if (e.button !== 0) return;

    // Set the files to be dragged
    setPendingFiles(files);
    
    // Capture drag image from the container
    const container = containerRef.current;
    if (container) {
      prepareDragImage(container).then(() => {
        // Small delay to ensure image is captured
        dragTimeoutRef.current = window.setTimeout(() => {
          triggerNativeDrag();
        }, 10);
      });
    } else {
      // No container, just trigger drag with default image
      dragTimeoutRef.current = window.setTimeout(() => {
        triggerNativeDrag();
      }, 10);
    }
  }, [files]);

  const handleMouseUp = useCallback(() => {
    if (dragTimeoutRef.current) {
      clearTimeout(dragTimeoutRef.current);
      dragTimeoutRef.current = null;
    }
  }, []);

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
