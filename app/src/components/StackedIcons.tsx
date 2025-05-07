import React, { useMemo } from 'react';
import { DynamicFileIcon } from './FileIcon';

interface File {
  preview?: string;
  name: string;
  path: string;
  size: number;
}

interface StackedIconsProps {
  files: File[];
  handleStackDragStart: (e: React.DragEvent<HTMLDivElement>) => void;
}

export const StackedIcons: React.FC<StackedIconsProps> = ({ files, handleStackDragStart }) => {
  const stackedIcons = useMemo(() => {
    return files.slice(-5).map((file, index) => {
      const rotation = Math.random() * 10 - 5;
      const translateX = Math.random() * 10 - 5;
      const translateY = Math.random() * 10 - 5;
      const zIndex = files.length - index;
    
      return (
        <div
          key={index}
          className="absolute inset-0 flex items-center justify-center overflow-hidden"
          style={{
            transform: `rotate(${rotation}deg) translate(${translateX}px, ${translateY}px)`,
            zIndex,
          }}
          draggable
          onDragStart={handleStackDragStart}
        >
          {file.preview && file.size < 5 * 1024 * 1024 ? (
            <img 
              src={file.preview} 
              alt={file.name} 
              className="w-full h-full object-cover"
              loading="lazy"
            />
          ) : (
            <DynamicFileIcon filePath={file.path} />
          )}
        </div>
      );
    });
  }, [files, handleStackDragStart]);

  return <div className="relative w-full h-full">{stackedIcons}</div>;
};
