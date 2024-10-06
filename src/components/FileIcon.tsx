import React, { useState, useEffect } from 'react';
import { FileIcon } from 'lucide-react';
import { useFileManagement } from '../hooks/useFileManagement';

interface DynamicFileIconProps {
  filePath: string;
}

export const DynamicFileIcon: React.FC<DynamicFileIconProps> = ({ filePath, ...props }) => {
  const [iconBase64, setIconBase64] = useState<string | null>(null);
  const { getFileIcon } = useFileManagement();

  useEffect(() => {
    const fetchIcon = async () => {
      try {
        const base64Icon = await getFileIcon(filePath);
        setIconBase64(base64Icon);
      } catch (error) {
        console.error('Error fetching file icon:', error);
      }
    };

    fetchIcon();
  }, [filePath, getFileIcon]);

  if (iconBase64) {
    return <img {...props} className="h-full w-full" src={`data:image/png;base64,${iconBase64}`} alt="File icon" />;
  }

  // Fallback to default FileIcon if no custom icon is available
  return <FileIcon className="h-6 w-6 text-blue-500" />;
};