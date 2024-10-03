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

interface FilePreview {
  id: number;
  name: string;
  preview: string;
  type: 'file' | 'folder';
  size: string;
}

function App() {
  const [isDragging, setIsDragging] = useState(false);
  const [files, setFiles] = useState<FilePreview[]>([]);
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const [isRenameDialogOpen, setIsRenameDialogOpen] = useState(false);
  const [fileToRename, setFileToRename] = useState<FilePreview | null>(null);
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
            name: path.split('/').pop() || '',
            preview: '',
            type: 'file' as const,
            size: '0 KB' // You might want to get the actual file size
          }));
          handleNewFiles(newFiles);
        }
      });
    };

    setupFileListener();
  }, []);

  const handleNewFiles = (newFiles: FilePreview[]) => {
    setFiles(prevFiles => {
      const uniqueNewFiles = newFiles.filter(newFile => 
        !prevFiles.some(existingFile => existingFile.name === newFile.name)
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
      size: '0 KB' // You might want to get the actual file size
    }));

    handleNewFiles(newFiles);
  }, []);

  const toggleDropdown = () => setIsDropdownOpen(!isDropdownOpen);

  const removeFile = (id: number) => {
    setFiles(files.filter(file => file.id !== id));
  };

  const openRenameDialog = (file: FilePreview) => {
    setFileToRename(file);
    setNewFileName(file.name);
    setIsRenameDialogOpen(true);
  };

  const renameFile = () => {
    if (!fileToRename) return;

    const oldExtension = fileToRename.name.split('.').pop();
    const newExtension = newFileName.split('.').pop();

    if (oldExtension !== newExtension) {
      console.error("Cannot change file extension");
      return;
    }

    setFiles(files.map(file => 
      file.id === fileToRename.id ? { ...file, name: newFileName } : file
    ));
    setIsRenameDialogOpen(false);
    console.log("File renamed successfully");
  };

  const handleDragStart = useCallback((e: React.DragEvent<HTMLDivElement>, file: FilePreview) => {
    e.preventDefault();
    e.stopPropagation();

    // Prepare the drag data
    e.dataTransfer.setData('text/plain', file.name);
    e.dataTransfer.effectAllowed = 'copy';

    // Call the Tauri API to initiate the OS-level drag
    invoke('start_drag', { filePath: file.preview })
      .then(() => console.log('Drag started successfully'))
      .catch((error) => console.error('Error starting drag:', error));
  }, []);

  const handleMultiFileDragStart = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();

    // Prepare the drag data for multiple files
    const fileNames = files.map(file => file.name).join('\n');
    e.dataTransfer.setData('text/plain', fileNames);
    e.dataTransfer.effectAllowed = 'copy';

    // Call the Tauri API to initiate the OS-level drag for multiple files
    invoke('start_multi_drag', { filePaths: files.map(file => file.preview) })
      .then(() => console.log('Multi-file drag started successfully'))
      .catch((error) => console.error('Error starting multi-file drag:', error));
  }, [files]);

  const renderFilePreview = () => {
    if (files.length === 0) {
      return (
        <div className="flex items-center justify-center h-full">
          <p className="text-black text-lg font-medium">Drop files here</p>
        </div>
      );
    } else {
      return (
        <div className="flex flex-col items-center justify-center h-full">
          <div 
            className="w-40 h-40 bg-white rounded-lg mb-6 flex items-center justify-center shadow-md cursor-move"
            draggable
            onDragStart={handleMultiFileDragStart}
          >
            <FileIcon className="h-16 w-16 text-[#0078d4]" />
          </div>
          <Button
            variant="outline"
            onClick={toggleDropdown}
            className="flex items-center space-x-2 bg-white text-black border-[#d1d1d1] hover:bg-[#f9f9f9] rounded-md"
          >
            <span>{files.length} Files</span>
            {isDropdownOpen ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
          </Button>
        </div>
      );
    }
  };

  return (
    <div className="fixed inset-0 bg-[#f3f3f3] text-black flex items-center justify-center">
      <div
        className={`w-full h-full max-w-md mx-auto flex flex-col ${
          isDragging ? 'ring-2 ring-[#0078d4] ring-opacity-60' : ''
        }`}
        onDragEnter={handleDragEnter}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        <div className="flex justify-between p-4">
          <Button variant="ghost" size="icon" className="text-black hover:bg-black/10 rounded-full">
            <X className="h-6 w-6" />
          </Button>
          <Button variant="ghost" size="icon" className="text-black hover:bg-black/10 rounded-full">
            <MoreVertical className="h-6 w-6" />
          </Button>
        </div>
        <div className="flex-grow flex flex-col items-center justify-center p-4">
          {renderFilePreview()}
        </div>
      </div>
      {isDropdownOpen && (
        <div className="fixed inset-x-0 bottom-0 bg-white rounded-t-xl shadow-lg transition-all duration-300 ease-in-out"
             style={{ height: 'calc(100% - 4rem)' }}>
          <div className="p-4 border-b border-[#e6e6e6] flex justify-between items-center">
            <h2 className="text-lg font-semibold text-black">{files.length} Files</h2>
            <Button variant="ghost" size="sm" onClick={toggleDropdown} className="text-black hover:bg-black/10 rounded-full">
              <X className="h-4 w-4" />
            </Button>
          </div>
          <ScrollArea className="h-[calc(100%-4rem)]">
            <ul className="p-4 space-y-4">
              {files.map((file) => (
                <li
                  key={file.id}
                  className="flex items-center space-x-4 hover:bg-[#f9f9f9] p-2 rounded-md"
                  draggable
                  onDragStart={(e: React.DragEvent<HTMLLIElement>) => handleDragStart(e as any, file)}
                >
                  <div className="w-10 h-10 bg-[#f0f0f0] rounded-md flex items-center justify-center">
                    <FileIcon className="h-6 w-6 text-[#0078d4]" />
                  </div>
                  <div className="flex-grow">
                    <p className="text-sm font-medium truncate text-black">{file.name}</p>
                    <p className="text-xs text-[#767676]">{file.size}</p>
                  </div>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" size="sm" className="text-black hover:bg-black/10 rounded-full">
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
            <Button type="submit" onClick={renameFile} className="bg-[#0078d4] hover:bg-[#006cbd] text-white rounded-md">
              Save changes
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

export default App;