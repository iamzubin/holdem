import { DynamicFileIcon } from "@/components/FileIcon";
import { Button } from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { useFileManagement } from "@/hooks/useFileManagement";
import { handleMultiFileDragStart } from "@/lib/fileUtils";
import { invoke } from "@tauri-apps/api/core";
import { MoreHorizontal, Trash2, X } from 'lucide-react';
import React, { useEffect, useState, useCallback } from "react";
import { Toaster, useSonner } from "sonner";

const PopupWindow: React.FC = () => {
  const { files, removeFile } = useFileManagement();
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [hasInteracted, setHasInteracted] = useState(false);
  const toast = useSonner();

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

  return (
    <div className="fixed inset-0 bg-gray-900 text-white p-4 overflow-auto">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-xl font-bold">Files</h2>
        <Button variant="ghost" size="sm" className="text-gray-400 hover:bg-red-500 hover:text-white rounded-full" onClick={() => invoke('close_popup_window')}>
          <X className="h-4 w-4" />
        </Button>
      </div>
      <div className="space-y-2">
        {files.map(file => (
          <div
            key={file.id}
            className="flex items-center space-x-2 p-2 rounded-md hover:bg-gray-800"
            draggable
            onDragStart={(e) => handleDragStart(e, file)}
          >
            <input
              type="checkbox"
              checked={selectedFiles.has(file.id?.toString() || '')}
              onChange={() => toggleFileSelection(file.id || '')}
              className="mr-1"
            />
            <div className="w-8 h-8 bg-gray-700 rounded-md flex items-center justify-center overflow-hidden">
              {file.preview ? (
                <img src={file.preview} alt={file.name} className="w-full h-full object-cover" />
              ) : (
                <DynamicFileIcon icon={file.icon} />
              )}
            </div>
            <div className="flex-grow">
              <p className="text-xs font-medium truncate">{file.name}</p>
            </div>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="icon" className="text-gray-400 hover:bg-gray-700 rounded-full p-1">
                  <MoreHorizontal className="h-3 w-3" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent className="bg-gray-800 text-white border-gray-700">
                <DropdownMenuItem onSelect={() => removeFile(file.id)} className="text-red-500 hover:bg-red-900">
                  <Trash2 className="mr-2 h-4 w-4" />
                  <span>Delete</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        ))}
      </div>
      <Toaster />
    </div>
  );
};

export default PopupWindow;