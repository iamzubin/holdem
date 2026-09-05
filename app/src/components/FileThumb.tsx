import React from 'react';
import { FilePreview } from '@/types';
import { DynamicFileIcon } from './FileIcon';

interface FileThumbProps {
  file: FilePreview;
  className?: string;
}

/** Canonical thumbnail branch: preview image when present, OS icon otherwise. */
export const FileThumb: React.FC<FileThumbProps> = ({ file, className }) => {
  if (file.preview) {
    return <img src={file.preview} alt={file.name} loading="lazy" className={className ?? 'h-full w-full object-cover'} />;
  }
  return <DynamicFileIcon file={file} />;
};
