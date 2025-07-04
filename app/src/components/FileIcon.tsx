import React, { useState, useEffect } from 'react';
import { FileIcon } from 'lucide-react';
import { useFileManagement } from '../hooks/useFileManagement';
import { File } from './StackedIcons';

interface DynamicFileIconProps {
  file: File
}

export const DynamicFileIcon: React.FC<DynamicFileIconProps> = ({ file, ...props }) => {
  const [iconBase64, setIconBase64] = useState<string | null>(null);
  const { getFileIcon } = useFileManagement();
  const [isVisible, setIsVisible] = useState(false);
  const iconRef = React.useRef<HTMLDivElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setIsVisible(true);
          observer.disconnect();
        }
      },
      { threshold: 0.1 }
    );

    if (iconRef.current) {
      observer.observe(iconRef.current);
    }

    return () => {
      observer.disconnect();
    };
  }, []);

  useEffect(() => {
    if (isVisible) {
      const fetchIcon = async () => {
        try {
          const base64Icon = await getFileIcon(file.path);
          setIconBase64(base64Icon);
        } catch (error) {
          console.error('Error fetching file icon:', error);
        }
      };

      fetchIcon();
    }
  }, [isVisible, file.path, getFileIcon]);

  return (
    <div ref={iconRef} {...props}>
      {iconBase64 ? (
        <img className="h-full w-full" src={`data:image/png;base64,${iconBase64}`} alt="File icon" />
      ) : (
        <FileIcon className="h-6 w-6 text-blue-500" />
      )}
    </div>
  );
};