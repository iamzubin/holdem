"use client"

import { useState, useCallback, useEffect, useRef } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { X, ChevronDown, ChevronUp, FileIcon, MoreHorizontal } from 'lucide-react';
import { formatFileSize } from "@/lib/utils";
import { handleDragStart, handleMultiFileDragStart } from "@/lib/fileUtils";
import { minimizeWindow, maximizeWindow, closeWindow } from "@/lib/windowUtils";
import { DynamicFileIcon } from "@/components/FileIcon";
import { useFileManagement } from "@/hooks/useFileManagement";
import { FilePreview } from "@/types";

function App() {
  const [isDragging, setIsDragging] = useState(false);
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const [isRenameDialogOpen, setIsRenameDialogOpen] = useState(false);
  const [fileToRename, setFileToRename] = useState<FilePreview | null>(null);
  const [newFileName, setNewFileName] = useState("");
  const listenerSetup = useRef(false);

  const { files, handleNewFiles, removeFile, renameFile } = useFileManagement();

  useEffect(() => {
    if (listenerSetup.current) return;
    listenerSetup.current = true;

    const setupFileListener = async () => {
      const webview = await getCurrentWebview();
      await webview.onDragDropEvent((event) => {
        if (event.payload.type === 'drop') {
          const newFiles = event.payload.paths.map((path, index) => ({
            id: Date.now() + index,
            name: path.split('/').pop() || '',
            preview: '',
            type: 'file' as const,
            size: 0,
            path: path,
            icon: ''
          }));
          handleNewFiles(newFiles);
        }
      });
    };

    setupFileListener();
  }, [handleNewFiles]);

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

  const handleDrop = useCallback((e: React.DragEvent<HTMLDivElement>) => {
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
      icon: ''
    }));

    handleNewFiles(newFiles);
  }, [handleNewFiles]);

  const toggleDropdown = () => setIsDropdownOpen(!isDropdownOpen);

  const openRenameDialog = (file: FilePreview) => {
    setFileToRename(file);
    setNewFileName(file.name);
    setIsRenameDialogOpen(true);
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

  const renderFilePreview = () => {
    if (files.length === 0) {
      return (
        <div className="flex items-center justify-center h-full">
          <p className="text-gray-600 text-lg font-medium">Drop files here</p>
        </div>
      );
    } else {
      return (
        <div className="flex flex-col items-center justify-center h-full">
          <div 
            className="w-40 h-40 bg-white rounded-lg mb-6 flex items-center justify-center shadow-md cursor-move"
            draggable
            onDragStart={(e) => handleMultiFileDragStart(e, files)}
          >
            <FileIcon className="h-16 w-16 text-blue-500" />
          </div>
          <Button
            variant="outline"
            onClick={toggleDropdown}
            className="flex items-center space-x-2 bg-white text-gray-800 border-gray-200 hover:bg-gray-100 rounded-full px-4 py-2"
          >
            <span>{files.length} Files</span>
            {isDropdownOpen ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
          </Button>
        </div>
      );
    }
  };

  return (
    <div className="fixed inset-0 bg-[#f3f3f3] text-gray-800 flex flex-col">
      <div className="flex justify-between items-center p-2 bg-[#f9f9f9] select-none" data-tauri-drag-region>
        <div className="flex items-center space-x-2 flex-grow" data-tauri-drag-region>
          <img src="/path-to-your-app-icon.png" alt="App Icon" className="w-4 h-4" />
          <span className="text-sm font-semibold">Your App Name</span>
        </div>
        <div className="flex space-x-2">
          <Button variant="ghost" size="icon" className="text-gray-600 hover:bg-gray-200" onClick={minimizeWindow}>
            <span className="w-4 h-4 flex items-center justify-center">&#8211;</span>
          </Button>
          <Button variant="ghost" size="icon" className="text-gray-600 hover:bg-gray-200" onClick={maximizeWindow}>
            <span className="w-4 h-4 flex items-center justify-center">&#9633;</span>
          </Button>
          <Button variant="ghost" size="icon" className="text-gray-600 hover:bg-red-500 hover:text-white" onClick={closeWindow}>
            <X className="h-4 w-4" />
          </Button>
        </div>
      </div>
      <div className="flex-grow flex flex-col items-center justify-center p-4"
           onDragEnter={handleDragEnter}
           onDragOver={handleDragOver}
           onDragLeave={handleDragLeave}
           onDrop={handleDrop}>
        {renderFilePreview()}
      </div>
      {isDropdownOpen && (
        <div className="fixed inset-x-0 bottom-0 bg-white rounded-t-xl shadow-lg transition-all duration-300 ease-in-out"
             style={{ height: 'calc(100% - 4rem)' }}>
          <div className="p-4 border-b border-gray-200 flex justify-between items-center">
            <h2 className="text-lg font-semibold text-gray-800">{files.length} Files</h2>
            <Button variant="ghost" size="sm" onClick={toggleDropdown} className="text-gray-600 hover:bg-gray-200 rounded-full">
              <X className="h-4 w-4" />
            </Button>
          </div>
          <ScrollArea className="h-[calc(100%-4rem)]">
            <ul className="p-4 space-y-4">
              {files.map((file) => (
                <li
                  key={file.id}
                  className="flex items-center space-x-4 hover:bg-gray-100 p-2 rounded-md"
                  draggable
                  onDragStart={(e: React.DragEvent<HTMLLIElement>) => handleDragStart(e as any, file)}
                >
                  <div className="w-10 h-10 bg-gray-200 rounded-md flex items-center justify-center">
                    <DynamicFileIcon icon={file.icon} />
                  </div>
                  <div className="flex-grow">
                    <p className="text-sm font-medium truncate text-gray-800">{file.name}</p>
                    <p className="text-xs text-gray-500">{formatFileSize(file.size)}</p>
                  </div>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" size="sm" className="text-gray-600 hover:bg-gray-200 rounded-full">
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      <DropdownMenuItem onSelect={() => openRenameDialog(file)}>Rename</DropdownMenuItem>
                      <DropdownMenuItem onSelect={() => removeFile(file.id)}>Remove</DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </li>
              ))}
            </ul>
          </ScrollArea>
        </div>
      )}
      <Dialog open={isRenameDialogOpen} onOpenChange={setIsRenameDialogOpen}>
        <DialogContent className="bg-white text-black rounded-lg">
          <DialogHeader>
            <DialogTitle className="text-black">Rename File</DialogTitle>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="name" className="text-right text-black">
                Name
              </Label>
              <Input
                id="name"
                value={newFileName}
                onChange={(e) => setNewFileName(e.target.value)}
                className="col-span-3 border-[#d1d1d1] focus:border-[#0078d4] focus:ring-[#0078d4]"
              />
            </div>
          </div>
          <DialogFooter>
            <Button type="submit" onClick={handleRename} className="bg-[#0078d4] hover:bg-[#006cbd] text-white rounded-md">
              Save changes
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

export default App;