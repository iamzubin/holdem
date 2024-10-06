"use client"

import { DynamicFileIcon } from "@/components/FileIcon";
import { Button } from "@/components/ui/button";
import { useFileManagement } from "@/hooks/useFileManagement";
import { handleMultiFileDragStart } from "@/lib/fileUtils";
import { closeWindow } from "@/lib/windowUtils";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { readFile, stat } from "@tauri-apps/plugin-fs";
import { ChevronDown, X, Download, Clipboard, Copy, Trash } from 'lucide-react';
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { FilePreview } from "@/types";
import { getFileExtension } from "./lib/utils";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { DialogClose } from "@radix-ui/react-dialog";

function App() {
  const [isDragging, setIsDragging] = useState(false);
  const listenerSetup = useRef(false);
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [modalPosition, setModalPosition] = useState({ x: 0, y: 0 });

  const { files, addFiles, getFileIcon } = useFileManagement();

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
                const binaryData = await readFile(path);
                const blob = new Blob([binaryData], { type: `image/${extension}` });
                preview = URL.createObjectURL(blob);
              }

              // Call getFileIcon and log the result
              try {
                const iconBase64 = await getFileIcon(path);
                console.log(`Icon for ${path}:`, iconBase64);
              } catch (iconError) {
                console.error(`Error getting icon for ${path}:`, iconError);
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
  }, [addFiles, getFileIcon]);

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
    console.log("handleDragOver")
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
    invoke('open_popup_window').catch((err) => console.error(err));
  };

  const handleStackDragStart = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    handleMultiFileDragStart(e, files);
  }, [files]);

  const stackedIcons = useMemo(() => {
    return files.slice(-10).map((file, index) => {
      const rotation = Math.random() * 10 - 5;
      const translateX = Math.random() * 10 - 5;
      const translateY = Math.random() * 10 - 5;
      const zIndex = files.length - index;
    
      return (
        <div
          key={file.id}
          className="absolute rounded-md shadow-sm flex items-center justify-center overflow-hidden h-[40px] w-[40px]"
          style={{
            transform: `rotate(${rotation}deg) translate(${translateX}px, ${translateY}px)`,
            zIndex,
          }}
          draggable
          onDragStart={handleStackDragStart}
        >
          {file.preview ? (
            <img src={file.preview} alt={file.name} />
          ) : (
              <DynamicFileIcon filePath={file.path} />
          )}
        </div>
      );
    });
  }, [files, handleStackDragStart]);

  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setModalPosition({ x: e.clientX, y: e.clientY });
    setIsModalOpen(true);
  }, []);

  return (
    <div 
      className="fixed inset-0 text-white flex flex-col bg-black p-2"
      onContextMenu={handleContextMenu}
    >
      {/* Handle and Title Bar */}
      <div className="relative flex justify-end items-center h-5" data-tauri-drag-region>
        <div className="absolute left-1/2 transform -translate-x-1/2 top-1/2 -translate-y-1/2">
          <div className="w-10 h-0.5 bg-gray-400 rounded-full"></div>
        </div>
        <Button variant="ghost" size="icon" className="text-gray-400 hover:bg-red-500 hover:text-white rounded h-5 w-5" onClick={closeWindow}>
          <X className="h-4 w-4" />
        </Button>
      </div>
      {/* Main Content */}
      <div className="flex-grow flex flex-col items-center justify-center space-y-1"
           onDragEnter={handleDragEnter}
           onDragOver={handleDragOver}
           onDragLeave={handleDragLeave}
           onDrop={handleDrop}>
        {files.length > 0 ? (
          <div className="relative w-20 h-20 flex items-center justify-center" draggable onDragStart={handleStackDragStart}>
            {stackedIcons}
          </div>
        ) : (
          <div className="flex flex-col items-center">
            <Download className="h-5 w-5" />
            <span className="text-[8px]">Drop here</span>
          </div>
        )}
      </div>

      {/* Dropdown Button at the Bottom */}
      <div className="flex justify-center items-center mt-1">
        <Button
          onClick={openPopup}
          className="flex items-center justify-between text-white border-gray-600 hover:bg-gray-600 rounded px-2 py-0.5 text-[10px] w-20"
        >
          <span>{files.length} file{files.length !== 1 ? 's' : ''}</span>
          <ChevronDown className="h-2 w-2 ml-1" />
        </Button>
      </div>

      <Dialog open={isModalOpen} onOpenChange={setIsModalOpen}>
        <DialogContent 
          className="bg-black text-white rounded-md p-0 mt-2 w-[90vw]"
        >
          <div className="flex flex-col items-start">
            {files.length > 0 ? (
              <>
                <Button 
                  className="w-full text-left justify-start hover:bg-gray-700 transition-colors"
                  variant="ghost"
                >
                  <Copy className="h-4 w-4 mr-2" />
                  Copy
                </Button>
                <Button 
                  className="w-full text-left justify-start hover:bg-gray-700 transition-colors"
                  variant="ghost"
                >
                  <Clipboard className="h-4 w-4 mr-2" />
                  Paste
                </Button>
                <div className="w-full h-[1px] bg-gray-600 mx-2"></div>
                <Button 
                  className="w-full text-left justify-start hover:bg-gray-700 transition-colors"
                  variant="ghost"
                >
                  <X  className="h-4 w-4 mr-2" />
                  Clear
                </Button>
              </>
            ) : (
              <Button 
                className="w-full text-left justify-start  hover:bg-gray-700 transition-colors"
                variant="ghost"
              >
                {/* paste icon */}
                <Clipboard className="h-4 w-4 mr-2" />
                Paste
              </Button>
            )}
          </div>
          <DialogClose asChild>
          </DialogClose>
        </DialogContent>
      </Dialog>
    </div>
  );
}

export default App;