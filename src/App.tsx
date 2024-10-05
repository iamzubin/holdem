"use client"

import { useState, useCallback, useEffect, useRef, useMemo } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { X, ChevronDown, ChevronUp, FileIcon, MoreHorizontal, Trash2 } from 'lucide-react';
import { handleDragStart, handleMultiFileDragStart } from "@/lib/fileUtils";
import { minimizeWindow, maximizeWindow, closeWindow } from "@/lib/windowUtils";
import { DynamicFileIcon } from "@/components/FileIcon";
import { useFileManagement } from "@/hooks/useFileManagement";
import { FilePreview } from "@/types";
import { stat, readFile } from "@tauri-apps/plugin-fs";


function App() {
  const [isDragging, setIsDragging] = useState(false);
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const [isRenameDialogOpen, setIsRenameDialogOpen] = useState(false);
  const [fileToRename, setFileToRename] = useState<FilePreview | null>(null);
  const [newFileName, setNewFileName] = useState("");
  const listenerSetup = useRef(false);
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());

  const { files, handleNewFiles, removeFile, renameFile } = useFileManagement();

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
          handleNewFiles(validNewFiles);
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

  const getFileExtension = (filename: string): string => {
    const ext = filename.split('.').pop()?.toLowerCase();
    if (!ext) return 'file';
    const imageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'svg'];
    const pdfExtensions = ['pdf'];
    if (imageExtensions.includes(ext)) return 'image';
    if (pdfExtensions.includes(ext)) return 'pdf';
    return 'file';
  };

  const stackedIcons = useMemo(() => {
    return files.slice(-3).map((file, index) => {
      const rotation = Math.random() * 10 - 5; // Random rotation between -5 and 5 degrees
      const translateX = Math.random() * 10 - 5; // Random X offset between -5 and 5 pixels
      const translateY = Math.random() * 10 - 5; // Random Y offset between -5 and 5 pixels
      const zIndex = files.length - index;

      return (
        <div
          key={file.id}
          className="absolute w-32 h-32 rounded-lg shadow-md flex items-center justify-center overflow-hidden"
          style={{
            transform: `rotate(${rotation}deg) translate(${translateX}px, ${translateY}px)`,
            zIndex,
          }}
        >
          {file.preview ? (
            <img src={file.preview} alt={file.name} className="w-full h-full object-cover" />
          ) : (
            <div className="w-full h-full bg-white flex items-center justify-center">
              <DynamicFileIcon icon={file.icon}/>
            </div>
          )}
        </div>
      );
    });
  }, [files]);

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
  const handleSelectedFilesDragStart = (e: React.DragEvent<HTMLDivElement>) => {
    const selectedFilesList = files.filter(file => selectedFiles.has(file.id.toString()));
    handleMultiFileDragStart(e, selectedFilesList);
  };

  const deleteSelectedFiles = () => {
    selectedFiles.forEach(fileId => removeFile(parseInt(fileId)));
    setSelectedFiles(new Set());
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
            className="relative w-40 h-40 mb-6 cursor-move"
            draggable
            onDragStart={(e) => handleMultiFileDragStart(e, files)}
          >
            {stackedIcons}
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
        <div className="fixed inset-x-0 bottom-0 bg-white rounded-t-xl shadow-lg transition-all duration-300 ease-in-out z-50"
             style={{ height: 'calc(100% - 4rem)' }}>
          <div className="p-4 border-b border-gray-200 flex justify-between items-center">
            <h2 className="text-lg font-semibold text-gray-800">{files.length} Files</h2>
            <div className="flex items-center space-x-2">
              <Button
                variant="destructive"
                size="sm"
                onClick={deleteSelectedFiles}
                disabled={selectedFiles.size === 0}
                className="bg-red-500 hover:bg-red-600 text-white rounded-md"
              >
                <Trash2 className="h-4 w-4 mr-2" />
                Delete Selected
              </Button>
              <Button variant="ghost" size="sm" onClick={toggleDropdown} className="text-gray-600 hover:bg-gray-200 rounded-full">
                <X className="h-4 w-4" />
              </Button>
            </div>
          </div>
          <ScrollArea className="h-[calc(100%-4rem)]">
            <ul className="p-4 space-y-4">
              {files.map((file) => (
                <li
                  key={file.id}
                  className={`flex items-center space-x-4 hover:bg-gray-100 p-2 rounded-md ${
                    selectedFiles.has(file.id.toString()) ? 'bg-blue-100' : ''
                  }`}
                  draggable={selectedFiles.has(file.id.toString())}
                  onDragStart={(e: React.DragEvent<HTMLLIElement>) => handleSelectedFilesDragStart(e)}
                >
                  <input
                    type="checkbox"
                    checked={selectedFiles.has(file.id.toString())}
                    onChange={() => toggleFileSelection(file.id.toString())}
                    className="mr-2"
                  />
                  <div className="w-10 h-10 bg-gray-200 rounded-md flex items-center justify-center overflow-hidden">
                    {file.preview ? (
                      <img src={file.preview} alt={file.name} className="w-full h-full object-cover" />
                    ) : (
                      <DynamicFileIcon icon={file.icon} />
                    )}
                  </div>
                  <div className="flex-grow">
                    <p className="text-sm font-medium truncate text-gray-800">{file.path.split('\\').pop()}</p>
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