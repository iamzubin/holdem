import { useState, useEffect, useRef, useCallback } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { invoke } from "@tauri-apps/api/core";
import { Toaster, toast } from "sonner";
import { Card, CardContent } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { FileIcon, FolderIcon, CopyIcon } from "lucide-react";
import { useDropzone } from "react-dropzone";

import "./App.css";

function App() {
  const [files, setFiles] = useState<{ name: string; type: 'file' | 'folder'; path: string }[]>([]);
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const listenerSetup = useRef(false);

  useEffect(() => {
    if (listenerSetup.current) return;
    listenerSetup.current = true;

    const setupFileListener = async () => {
      const webview = await getCurrentWebview();
      await webview.onDragDropEvent((event) => {
        console.log("event", event);
        if (event.payload.type === 'drop') {
          handleFileDrop(event.payload.paths);
        }
      });
    };

    setupFileListener();
  }, []);

  const handleFileDrop = async (paths: string[]) => {
    try {
      const storedPaths = await invoke<string[]>("handle_file_drop", { paths });
      const newFiles = storedPaths.map(path => ({
        name: path.split('/').pop() || '',
        type: 'file' as const,
        path: path
      }));
      setFiles(prevFiles => {
        const uniqueNewFiles = newFiles.filter(newFile => 
          !prevFiles.some(existingFile => existingFile.path === newFile.path)
        );
        if (uniqueNewFiles.length > 0) {
          toast.success(`Added ${uniqueNewFiles.length} new file(s)`);
        } else {
          toast.info("No new files added");
        }
        return [...prevFiles, ...uniqueNewFiles];
      });
    } catch (error) {
      toast.error(`Error storing files: ${error}`);
    }
  };

  const onDrop = useCallback((acceptedFiles: File[]) => {
    handleFileDropFromBrowser(acceptedFiles);
  }, []);

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    noClick: true,
  });

  const handleDragOver = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
  };

  const handleDrop = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    
    const items = e.dataTransfer.items;
    const files: File[] = [];
    
    for (let i = 0; i < items.length; i++) {
      if (items[i].kind === 'file') {
        const file = items[i].getAsFile();
        if (file) {
          files.push(file);
        }
      }
    }

    if (files.length > 0) {
      handleFileDropFromBrowser(files);
    }
  };

  const handleFileDropFromBrowser = async (files: File[]) => {
    console.log("handleFileDropFromBrowser", files);
    try {
      const fileDataPromises = files.map(file => 
        new Promise<string>((resolve, reject) => {
          if (file.webkitRelativePath) {
            // Local file drop
            resolve(file.webkitRelativePath);
          } else {
            // Browser file drop
            const reader = new FileReader();
            reader.onload = () => resolve(reader.result as string);
            reader.onerror = reject;
            reader.readAsDataURL(file);
          }
        })
      );

      const fileDataArray = await Promise.all(fileDataPromises);
      const storedPaths = await invoke<string[]>("handle_browser_file_drop", { files: fileDataArray });
      
      const newFiles = storedPaths.map(path => ({
        name: path.split('/').pop() || '',
        type: 'file' as const,
        path: path
      }));

      setFiles(prevFiles => {
        const uniqueNewFiles = newFiles.filter(newFile => 
          !prevFiles.some(existingFile => existingFile.path === newFile.path)
        );
        if (uniqueNewFiles.length > 0) {
          toast.success(`Added ${uniqueNewFiles.length} new file(s)`);
        } else {
          toast.info("No new files added");
        }
        return [...prevFiles, ...uniqueNewFiles];
      });

      // Log full file paths or names
      files.forEach(file => console.log("Full file path or name:", file.webkitRelativePath || file.name));
    } catch (error) {
      toast.error(`Error storing files: ${error}`);
    }
  };

  const handleFileSelection = (path: string) => {
    setSelectedFiles(prevSelected => {
      if (prevSelected.includes(path)) {
        return prevSelected.filter(p => p !== path);
      } else {
        return [...prevSelected, path];
      }
    });
  };

  const handleCopySelectedFiles = async () => {
    if (selectedFiles.length === 0) {
      toast.info("No files selected");
      return;
    }

    try {
      await invoke("copy_selected_files", { paths: selectedFiles });
      toast.success(`Copied ${selectedFiles.length} file(s)`);
      setSelectedFiles([]);
    } catch (error) {
      toast.error(`Error copying files: ${error}`);
    }
  };

  return (
    <div className="container mx-auto p-4">
      <Card className="w-full h-[calc(100vh-2rem)]">
        <CardContent className="p-6">
          <div 
            {...getRootProps()} 
            onDragOver={handleDragOver}
            onDrop={handleDrop}
            className={`border-2 border-dashed rounded-lg p-8 mb-4 h-64 flex items-center justify-center ${
              isDragActive ? 'border-blue-500 bg-blue-50' : 'border-gray-300'
            }`}
          >
            <input {...getInputProps()} />
            {isDragActive ? (
              <p className="text-xl">Drop the files here ...</p>
            ) : (
              <p className="text-xl">Drag 'n' drop some files here, or click to select files</p>
            )}
          </div>
          <ScrollArea className="h-[calc(100vh-24rem)]">
            <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
              {files.map((item, index) => (
                <Button
                  key={index}
                  variant={selectedFiles.includes(item.path) ? "default" : "outline"}
                  className="h-24 flex flex-col items-center justify-center"
                  onClick={() => handleFileSelection(item.path)}
                >
                  {item.type === 'folder' ? <FolderIcon className="h-8 w-8 mb-2" /> : <FileIcon className="h-8 w-8 mb-2" />}
                  <span className="text-sm truncate w-full text-center">{item.name}</span>
                </Button>
              ))}
            </div>
          </ScrollArea>
          <div className="mt-4 flex justify-between items-center">
            <div className="text-sm text-gray-500">
              {files.length} item(s), {selectedFiles.length} selected
            </div>
            <Button onClick={handleCopySelectedFiles} disabled={selectedFiles.length === 0}>
              <CopyIcon className="mr-2 h-4 w-4" /> Copy Selected Files
            </Button>
          </div>
        </CardContent>
      </Card>
      <Toaster />
    </div>
  );
}

export default App;