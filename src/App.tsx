"use client"

import { useState, useCallback, useEffect, useRef } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { X, MoreVertical, ChevronDown, ChevronUp, FileIcon, MoreHorizontal } from 'lucide-react';
import { invoke } from "@tauri-apps/api/core";
import { Window } from '@tauri-apps/api/window';

interface FileInfo {
  id: number;
  path: string;
  name: string;
  size: number;
  is_directory: boolean;
}

export default function Component() {
  const [isDragging, setIsDragging] = useState(false);
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const [isRenameDialogOpen, setIsRenameDialogOpen] = useState(false);
  const [fileToRename, setFileToRename] = useState<FileInfo | null>(null);
  const [newFileName, setNewFileName] = useState("");
  const listenerSetup = useRef(false);

  useEffect(() => {
    if (listenerSetup.current) return;
    listenerSetup.current = true;

    const setupFileListener = async () => {
      const webview = await getCurrentWebview();
      await webview.onDragDropEvent((event) => {
        if (event.payload.type === 'drop') {
          const newFiles = event.payload.paths.map((path, index) => ({
            id: Date.now() + index,
            path: path,
            name: path.split('/').pop() || '',
            size: 0,
            is_directory: false
          }));
          handleNewFiles(newFiles);
        }
      });
    };

    setupFileListener();
  }, []);

  const handleNewFiles = async (newFiles: FileInfo[]) => {
    setFiles(prevFiles => {
      const uniqueNewFiles = newFiles.filter(newFile => 
        !prevFiles.some(existingFile => existingFile.id === newFile.id)
      );
      if (uniqueNewFiles.length > 0) {
        console.log(`Added ${uniqueNewFiles.length} new file(s)`);
      } else {
        console.log("No new files added");
      }
      return [...prevFiles, ...uniqueNewFiles];
    });
  };

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

    const paths = Array.from(e.dataTransfer.files).map(file => file.path);
    try {
      const newFiles = await invoke<FileInfo[]>('handle_file_drop', { paths });
      handleNewFiles(newFiles);
    } catch (error) {
      console.error('Error handling file drop:', error);
    }
  }, []);

  const toggleDropdown = () => setIsDropdownOpen(!isDropdownOpen);

  const removeFile = async (id: number) => {
    try {
      await invoke('remove_file', { id });
      setFiles(files.filter(file => file.id !== id));
    } catch (error) {
      console.error('Error removing file:', error);
    }
  };

  const openRenameDialog = (file: FileInfo) => {
    setFileToRename(file);
    setNewFileName(file.name);
    setIsRenameDialogOpen(true);
  };

  const renameFile = async () => {
    if (!fileToRename) return;

    try {
      await invoke('rename_file', { id: fileToRename.id, newName: newFileName });
      setFiles(files.map(file => 
        file.id === fileToRename.id ? { ...file, name: newFileName } : file
      ));
      setIsRenameDialogOpen(false);
      console.log("File renamed successfully");
    } catch (error) {
      console.error('Error renaming file:', error);
    }
  };

  const handleDragStart = useCallback((e: React.DragEvent<HTMLDivElement>, file: FileInfo) => {
    e.preventDefault();
    e.stopPropagation();

    e.dataTransfer.setData('text/plain', file.name);
    e.dataTransfer.effectAllowed = 'copy';

    invoke('start_drag', { filePath: file.path })
      .then(() => console.log('Drag started successfully'))
      .catch((error) => console.error('Error starting drag:', error));
  }, []);

  const handleMultiFileDragStart = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();

    const fileNames = files.map(file => file.name).join('\n');
    e.dataTransfer.setData('text/plain', fileNames);
    e.dataTransfer.effectAllowed = 'copy';

    invoke('start_multi_drag', { filePaths: files.map(file => file.path) })
      .then(() => console.log('Multi-file drag started successfully'))
      .catch((error) => console.error('Error starting multi-file drag:', error));
  }, [files]);

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
            onDragStart={async (e) => {
              e.preventDefault();
              e.stopPropagation();
              const fileIds = files.map(file => file.id);
              try {
                await invoke('start_multi_drag', { fileIds });
                console.log('Multi-file drag started successfully');
              } catch (error) {
                console.error('Error starting multi-file drag:', error);
              }
            }}
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

  const minimizeWindow = () => {
    Window.getCurrent().minimize();
  };

  const maximizeWindow = () => {
    Window.getCurrent().toggleMaximize();
  };

  const closeWindow = () => {
    Window.getCurrent().close();
  };

  useEffect(() => {
    const fetchStoredFiles = async () => {
      try {
        const storedFiles = await invoke<FileInfo[]>('get_stored_files');
        setFiles(storedFiles);
      } catch (error) {
        console.error('Error fetching stored files:', error);
      }
    };

    fetchStoredFiles();
  }, []);

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
                    <FileIcon className="h-6 w-6 text-blue-500" />
                  </div>
                  <div className="flex-grow">
                    <p className="text-sm font-medium truncate text-gray-800">{file.name}</p>
                    <p className="text-xs text-gray-500">{file.size}</p>
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
        <DialogContent className="bg-white text-gray-800 rounded-lg">
          <DialogHeader>
            <DialogTitle className="text-gray-800">Rename File</DialogTitle>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="name" className="text-right text-gray-600">
                Name
              </Label>
              <Input
                id="name"
                value={newFileName}
                onChange={(e) => setNewFileName(e.target.value)}
                className="col-span-3 border-gray-300 focus:border-blue-500 focus:ring-blue-500"
              />
            </div>
          </div>
          <DialogFooter>
            <Button type="submit" onClick={renameFile} className="bg-blue-500 hover:bg-blue-600 text-white rounded-full">
              Save changes
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}