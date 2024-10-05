"use client"

import { DynamicFileIcon } from "@/components/FileIcon";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useFileManagement } from "@/hooks/useFileManagement";
import { handleMultiFileDragStart } from "@/lib/fileUtils";
import { closeWindow } from "@/lib/windowUtils";
import { FilePreview } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { BaseDirectory , readFile, stat } from "@tauri-apps/plugin-fs";
import { ChevronDown, X, Download } from 'lucide-react';
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

function App() {
  const [isDragging, setIsDragging] = useState(false);
  const [isRenameDialogOpen, setIsRenameDialogOpen] = useState(false);
  const [fileToRename, setFileToRename] = useState<FilePreview | null>(null);
  const [newFileName, setNewFileName] = useState("");
  const listenerSetup = useRef(false);
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());

  const { files, addFiles, removeFile, renameFile } = useFileManagement();

  useEffect(() => {
    if (listenerSetup.current) return;
    listenerSetup.current = true;

    const setupFileListener = async () => {
      const webview = await getCurrentWebview();
      await webview.onDragDropEvent(async (event) => {
        if (event.payload.type === 'drop') {
          const filePromises = event.payload.paths.map(async (path, index) => {
            try {
              const fileStat = await stat(path);
              const extension = path.split('.').pop()?.toLowerCase() || '';
              const isImage = ['jpg', 'jpeg', 'png', 'gif', 'bmp'].includes(extension);
              let preview = '';

              if (isImage) {
                console.log(path);
                const binaryData = await readFile(path);
                const blob = new Blob([binaryData], { type: `image/${extension}` });
                console.log(blob);
                preview = URL.createObjectURL(blob);
              }

              return {
                id: Date.now() + index,
                name: path.split('/').pop() || '',
                preview,
                type: fileStat.isFile ? 'file' : 'folder',
                size: fileStat.size,
                path: path,
                icon: fileStat.isFile ? getFileExtension(path.split('/').pop() || '') : 'folder'
              };
            } catch (error) {
              console.error(`Error fetching stats for file: ${path}`, error);
              return null;
            }
          });

          const newFiles = await Promise.all(filePromises);
          const validNewFiles = newFiles.filter(file => file !== null) as FilePreview[];
          addFiles(validNewFiles);
        }
      });
    };

    setupFileListener();
  }, [addFiles, listenerSetup, setIsDragging]);

  const handleDragEnter = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback(async (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const droppedFiles = Array.from(e.dataTransfer.files);
    const newFiles: FilePreview[] = droppedFiles.map((file, index) => ({
      id: Date.now() + index,
      name: file.name,
      preview: URL.createObjectURL(file),
      type: 'file',
      size: file.size,
      path: (file as any).path,
      icon: getFileExtension(file.name)
    }));

    addFiles(newFiles);
  }, [addFiles]);

  const openPopup = () => {
    // Invoke the command to open the popup window
    invoke('open_popup_window').catch((err) => console.error(err));
  };

  const handleRename = () => {
    if (!fileToRename) return;

    const oldExtension = fileToRename.name.split('.').pop();
    const newExtension = newFileName.split('.').pop();

    if (oldExtension !== newExtension) {
      console.error("Cannot change file extension");
      return;
    }

    renameFile(fileToRename.id, newFileName);
    setIsRenameDialogOpen(false);
    console.log("File renamed successfully");
  };

  const getFileExtension = (filename: string): string => {
    const ext = filename.split('.').pop()?.toLowerCase();
    if (!ext) return 'file';
    const imageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'svg'];
    const pdfExtensions = ['pdf'];
    if (imageExtensions.includes(ext)) return 'image';
    if (pdfExtensions.includes(ext)) return 'pdf';
    return 'file';
  };

  const handleStackDragStart = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    handleMultiFileDragStart(e, files);
  }, [files]);

  const stackedIcons = useMemo(() => {
    return files.slice(-3).map(async (file, index) => {
      const rotation = Math.random() * 10 - 5;
      const translateX = Math.random() * 10 - 5;
      const translateY = Math.random() * 10 - 5;
      const zIndex = files.length - index;
    
      const fileStat = await stat(file.path);
      const extension = file.path.split('.').pop()?.toLowerCase() || '';
      const isImage = ['jpg', 'jpeg', 'png', 'gif', 'bmp'].includes(extension);
      let preview = '';

      if (isImage) {
        console.log(file.path);
        const binaryData = await readFile(file.path);
        const blob = new Blob([binaryData], { type: `image/${extension}` });
        console.log(blob);
        preview = URL.createObjectURL(blob);
      }
      console.log(file.preview);
      return (
        <div
          key={file.id}
          className="absolute w-24 h-24 rounded-lg shadow-md flex items-center justify-center overflow-hidden"
          style={{
            transform: `rotate(${rotation}deg) translate(${translateX}px, ${translateY}px)`,
            zIndex,
          }}
          draggable
          onDragStart={handleStackDragStart}
        >
          {file.preview ? (
            <img src={file.preview} alt={file.name} className="w-full h-full object-cover" />
          ) : (
            <div className="w-full h-full bg-white flex items-center justify-center">
              <DynamicFileIcon icon={file.icon} />
            </div>
          )}
        </div>
      );
    });
  }, [files, handleStackDragStart]);

  const toggleFileSelection = (fileId: string) => {
    setSelectedFiles(prev => {
      const newSet = new Set(prev);
      if (newSet.has(fileId)) {
        newSet.delete(fileId);
      } else {
        newSet.add(fileId);
      }
      return newSet;
    });
  };

  return (
    <div className="fixed inset-0 text-white flex flex-col bg-black">
      {/* Minimal Title Bar */}
      <div className="flex justify-end items-center p-1" data-tauri-drag-region>
        <Button variant="ghost" size="sm" className="text-gray-400 hover:bg-red-500 hover:text-white rounded-md" onClick={closeWindow}>
          <X className="h-4 w-4" />
        </Button>
      </div>
      
      {/* Main Content */}
      <div className="flex-grow flex flex-col items-center  justify-center p-2"
           onDragEnter={handleDragEnter}
           onDragOver={handleDragOver}
           onDragLeave={handleDragLeave}
           onDrop={handleDrop}>
        <div className="flex flex-col items-center p-4 rounded-lg">
          {files.length > 0 ? (
            <div className="relative w-32 h-32 mb-4" draggable onDragStart={handleStackDragStart}>
              <DynamicFileIcon
                icon={files[0].icon}
              />
            </div>
          ) : (
            <div className="flex flex-col items-center space-x-2 mb-4">
              <Download className="h-5 w-5" />
              <span className="text-lg">Drop items here</span>
            </div>
          )}
        </div>
        <Button
            variant="outline"
            onClick={openPopup}
            className="flex items-center space-x-1 text-white border-gray-600 hover:bg-gray-600 rounded-full px-3 py-1 text-sm mt-4"
          >
            <span>{files.length} items</span>
            <ChevronDown className="h-3 w-3" />
          </Button>
      </div>
      
    </div>
  );
}

export default App;