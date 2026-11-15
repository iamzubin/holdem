import React from 'react';
import { FilePreview } from '@/types';
import { DynamicFileIcon } from './FileIcon';

interface FileThumbProps {
  file: FilePreview;
}

/** Shelf thumbnail: the OS thumbnail or icon, as Explorer shows it. */
export const FileThumb: React.FC<FileThumbProps> = ({ file }) => <DynamicFileIcon file={file} />;
