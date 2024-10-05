import { FileIcon, FolderIcon, ImageIcon } from 'lucide-react';
import React from 'react';

interface DynamicFileIconProps {
  icon: string;
}

export const DynamicFileIcon: React.FC<DynamicFileIconProps> = ({ icon }) => {
  // Determine which icon to render based on the icon prop
  const renderIcon = () => {
    switch (icon) {
      case 'folder':
        return <FolderIcon className="h-6 w-6 text-yellow-500" />;

      case 'image':
        return <ImageIcon className="h-6 w-6 text-green-500" />;
      // Add more cases for different file types as needed
      default:
        return <FileIcon className="h-6 w-6 text-blue-500" />;
    }
  };

  return renderIcon();
};