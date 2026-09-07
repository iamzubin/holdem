import React, { useState, useEffect, useRef } from 'react';
import { FileIcon } from 'lucide-react';
import { FilePreview } from '@/types';
import { getFileIconBase64 } from '@/lib/fileIcon';

interface DynamicFileIconProps {
  file: FilePreview;
}

export const DynamicFileIcon: React.FC<DynamicFileIconProps> = ({ file }) => {
  const [iconBase64, setIconBase64] = useState<string | null>(null);
  const [isVisible, setIsVisible] = useState(false);
  const iconRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const target = iconRef.current;
    if (!target) return;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setIsVisible(true);
          observer.disconnect();
        }
      },
      { threshold: 0.1 },
    );
    observer.observe(target);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (!isVisible) return;
    let cancelled = false;
    getFileIconBase64(file.path)
      .then((base64Icon) => {
        if (!cancelled) setIconBase64(base64Icon);
      })
      .catch((error) => console.error('Error fetching file icon:', error));
    return () => {
      cancelled = true;
    };
  }, [isVisible, file.path]);

  return (
    <div ref={iconRef}>
      {iconBase64 ? (
        <img className="h-full w-full" src={`data:image/png;base64,${iconBase64}`} alt="File icon" />
      ) : (
        <FileIcon className="h-6 w-6 text-blue-500" />
      )}
    </div>
  );
};