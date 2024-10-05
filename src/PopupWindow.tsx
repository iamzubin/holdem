// src/PopupWindow.tsx
import { DynamicFileIcon } from "@/components/FileIcon";
import { Button } from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { useFileManagement } from "@/hooks/useFileManagement";
import { handleMultiFileDragStart } from "@/lib/fileUtils";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { MoreHorizontal, Trash2, X } from 'lucide-react';
import React, { useEffect, useState } from "react";
import { Toaster, useSonner } from "sonner";

const PopupWindow: React.FC = () => {
  const { files, removeFile, addFiles } = useFileManagement();
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [hasInteracted, setHasInteracted] = useState(false);
  const toast = useSonner();

  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (!hasInteracted) {
        invoke('close_popup_window').catch((err) => console.error(err));
      }
    }, 3000); // 3 seconds

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
  }, [hasInteracted, removeFile, addFiles, toast, files]);

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

  const handleSelectedFilesDragStart = (e: React.DragEvent<HTMLDivElement>) => {
    const selectedFilesList = files.filter(file => selectedFiles.has(file.id.toString()));
    handleMultiFileDragStart(e, selectedFilesList);
  };

  const deleteSelectedFiles = () => {
    selectedFiles.forEach(fileId => removeFile(parseInt(fileId)));
    setSelectedFiles(new Set());
  };

  return (
    <div className="h-full bg-black w-full p-3 flex flex-col text-white">
      {/* Minimal Header */}
      <div className="flex justify-end items-center mb-3">
        <Button size="sm" onClick={() => invoke('close_popup_window')} className="text-gray-400 hover:bg-red-500 hover:text-white rounded-md p-1">
          <X className="h-4 w-4" />
        </Button>
      </div>

      {/* File List */}
      <div className="flex-grow overflow-auto">
        {files.length === 0 ? (
          <p className="text-gray-400 text-sm">No files available.</p>
        ) : (
          <ul className="space-y-1">
            {files.map((file) => (
              <li
                key={file.id}
                className={`flex items-center space-x-3 p-1 rounded-md hover:bg-gray-800 ${
                  selectedFiles.has(file.id?.toString() || '') ? 'bg-gray-700' : ''
                }`}
                draggable={selectedFiles.has(file.id?.toString() || '')}
                onDragStart={(e: React.DragEvent<HTMLLIElement>) => {
                  e.preventDefault();
                  e.stopPropagation();
                  handleSelectedFilesDragStart(e as unknown as React.DragEvent<HTMLDivElement>);
                }}
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
                    <DropdownMenuItem onSelect={() => {/* Implement Rename Functionality */}} className="hover:bg-gray-700">
                      Rename
                    </DropdownMenuItem>
                    <DropdownMenuItem onSelect={() => removeFile(file.id)} className="hover:bg-gray-700">
                      Remove
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </li>
            ))}
          </ul>
        )}
      </div>

      {/* Footer Actions */}
      <div className="mt-3 flex justify-end">
        <Button
          variant="destructive"
          onClick={deleteSelectedFiles}
          disabled={selectedFiles.size === 0}
          className="flex items-center space-x-1 bg-red-500 hover:bg-red-600 text-white rounded-md text-sm px-2 py-1"
        >
          <Trash2 className="h-3 w-3" />
          <span>Delete</span>
        </Button>
      </div>
      <Toaster />
    </div>
    
  );
};

export default PopupWindow;