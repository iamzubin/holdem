import { DynamicFileIcon } from "@/components/FileIcon";
import { Button } from "@/components/ui/button";
import { useFileManagement } from "@/hooks/useFileManagement";
import { handleMultiFileDragStart } from "@/lib/fileUtils";
import { invoke } from "@tauri-apps/api/core";
import { MoreHorizontal, List as ListIcon, Grid as GridIcon, Trash2 } from 'lucide-react';
import React, { useEffect, useState, useCallback, useRef } from "react";
import { Toaster } from "sonner";
import * as ContextMenu from '@radix-ui/react-context-menu';

const PopupWindow: React.FC = () => {
  const { files } = useFileManagement();
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [hasInteracted, setHasInteracted] = useState(false);
  const [viewMode, setViewMode] = useState<'list' | 'grid'>('list');
  const [lastSelectedFile, setLastSelectedFile] = useState<string | null>(null);
  const fileRefs = useRef<{ [key: string]: HTMLDivElement | null }>({});

  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (!hasInteracted) {
        invoke('close_popup_window').catch((err) => console.error(err));
      }
    }, 3000);

    const handleFocus = () => {
      clearTimeout(timeoutId);
      setHasInteracted(true);
    };

    const handleBlur = () => {
      if (hasInteracted) {
        invoke('close_popup_window').catch((err) => console.error(err));
      }
    };

    window.addEventListener('focus', handleFocus);
    window.addEventListener('blur', handleBlur);

    return () => {
      window.removeEventListener('focus', handleFocus);
      window.removeEventListener('blur', handleBlur);
      clearTimeout(timeoutId);
    };
  }, [hasInteracted]);

  const toggleFileSelection = (fileId: string | number) => {
    setSelectedFiles(prev => {
      const newSet = new Set(prev);
      const idString = fileId.toString();
      if (newSet.has(idString)) {
        newSet.delete(idString);
      } else {
        newSet.add(idString);
      }
      return newSet;
    });
  };

  const handleDragStart = useCallback((e: React.DragEvent<HTMLDivElement>, file: any) => {
    e.stopPropagation();
    if (selectedFiles.size > 0) {
      const selectedFileObjects = files.filter(f => selectedFiles.has(f.id.toString()));
      handleMultiFileDragStart(e, selectedFileObjects);
    } else {
      handleMultiFileDragStart(e, [file]);
    }
  }, [files, selectedFiles]);

  const handleFileClick = useCallback((fileId: string, event: React.MouseEvent) => {
    setSelectedFiles(prev => {
      const newSet = new Set(prev);
      if (event.shiftKey && lastSelectedFile) {
        const fileIds = files.map(f => f.id.toString());
        const startIndex = fileIds.indexOf(lastSelectedFile);
        const endIndex = fileIds.indexOf(fileId);
        const [start, end] = [Math.min(startIndex, endIndex), Math.max(startIndex, endIndex)];
        for (let i = start; i <= end; i++) {
          newSet.add(fileIds[i]);
        }
      } else if (event.ctrlKey || event.metaKey) {
        if (newSet.has(fileId)) {
          newSet.delete(fileId);
        } else {
          newSet.add(fileId);
        }
      } else {
        newSet.clear();
        newSet.add(fileId);
      }
      return newSet;
    });
    setLastSelectedFile(fileId);
  }, [files, lastSelectedFile]);

  const getTotalSize = (files: any[]): string => {
    const totalBytes = files.reduce((acc, file) => acc + file.size, 0);
    return formatFileSize(totalBytes);
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024 * 1024) {
      return `${(bytes / 1024).toFixed(1)} KB`;
    }
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const toggleViewMode = () => {
    setViewMode(prev => prev === 'list' ? 'grid' : 'list');
  };

  const handleRemoveSelectedFiles = useCallback(() => {
    // Implement the logic to remove selected files
    // This is a placeholder function - you'll need to implement the actual removal logic
    console.log('Removing selected files:', Array.from(selectedFiles));
    // After removing, clear the selection
    setSelectedFiles(new Set());
  }, [selectedFiles]);

  return (
    <div className="fixed inset-0 bg-black text-white p-2 overflow-auto">
      <div className="flex justify-between items-center mb-2">
        <div className="flex items-center space-x-2">
          <Button variant="ghost" size="sm" className="text-gray-400 hover:accent-foreground rounded-full p-1">
            <MoreHorizontal className="h-4 w-4" />
          </Button>
          <span className="text-xs">{files.length} items selected</span>
          <span className="text-xs text-gray-400">{getTotalSize(files)}</span>
        </div>
        <div className="flex space-x-2">
          <Button
            variant="ghost"
            size="sm"
            className={`text-gray-400 hover:accent-foreground rounded-full p-1 ${viewMode === 'list' ? 'bg-gray-600' : ''}`}
            onClick={toggleViewMode}
          >
            <ListIcon className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className={`text-gray-400 hover:accent-foreground rounded-full p-1 ${viewMode === 'grid' ? 'bg-gray-600' : ''}`}
            onClick={toggleViewMode}
          >
            <GridIcon className="h-4 w-4" />
          </Button>
        </div>
      </div>
      <ContextMenu.Root>
        <ContextMenu.Trigger className="flex-grow">
          <div className={viewMode === 'grid' ? 'grid grid-cols-2 gap-1' : 'space-y-1'}>
            {files.map(file => (
              <div
                key={file.id}
                ref={el => fileRefs.current[file.id] = el}
                className={`
                  ${viewMode === 'list'
                    ? 'flex items-center space-x-2 p-1 rounded-md hover:accent-foreground'
                    : 'flex flex-col items-center p-1 rounded-md hover:accent-foreground'
                  }
                  ${selectedFiles.has(file.id.toString()) ? 'bg-blue-500 bg-opacity-50' : ''}
                  cursor-pointer
                `}
                onClick={(e) => handleFileClick(file.id.toString(), e)}
                draggable
                onDragStart={(e) => handleDragStart(e, file)}
              >
                <div className={`
                  rounded-md flex items-center justify-center overflow-hidden
                  ${viewMode === 'list' ? 'w-8 h-8 flex-shrink-0' : 'w-12 h-12 mb-1'}
                `}>
                  {file.preview ? (
                    <img src={file.preview} alt={file.name} className="w-full h-full object-cover" />
                  ) : (
                    <DynamicFileIcon filePath={file.path} />
                  )}
                </div>
                <div className={`
                  ${viewMode === 'list' ? 'flex-grow min-w-0' : 'w-full text-center'}
                `}>
                  <p className="text-xs font-medium truncate" title={file.name}>{file.name}</p>
                  {viewMode === 'grid' && (
                    <span className="text-[10px] text-gray-400">{formatFileSize(file.size)}</span>
                  )}
                </div>
                {viewMode === 'list' && (
                  <span className="text-[10px] text-gray-400 flex-shrink-0">{formatFileSize(file.size)}</span>
                )}
              </div>
            ))}
          </div>
        </ContextMenu.Trigger>
        <ContextMenu.Portal>
          <ContextMenu.Content className="min-w-[200px] bg-gray-800 rounded-md overflow-hidden p-1">
            <ContextMenu.Item 
              className="text-sm text-white hover:bg-gray-700 rounded flex items-center px-2 py-1 cursor-pointer"
              onClick={handleRemoveSelectedFiles}
              disabled={selectedFiles.size === 0}
            >
              <Trash2 className="w-4 h-4 mr-2" />
              Remove Selected Files
            </ContextMenu.Item>
          </ContextMenu.Content>
        </ContextMenu.Portal>
      </ContextMenu.Root>
      <Toaster />
    </div>
  );
};

export default PopupWindow;