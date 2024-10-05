import React from 'react';
import { FileIcon, FolderIcon, ImageIcon, FilesIcon } from 'lucide-react';

interface DynamicFileIconProps {
  icon: string;
}

export const DynamicFileIcon: React.FC<DynamicFileIconProps> = ({ icon }) => {
  switch (icon) {
    case 'folder':
      return <FolderIcon className="h-6 w-6 text-yellow-500" />;
    case 'image':
      return <ImageIcon className="h-6 w-6 text-green-500" />;
    case 'pdf':
      return <FilesIcon className="h-6 w-6 text-red-500" />;
    default:
      return <FileIcon className="h-6 w-6 text-blue-500" />;
  }
};