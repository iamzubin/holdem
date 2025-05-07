"use client"

import { DynamicFileIcon } from "@/components/FileIcon";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { useFileManagement } from "@/hooks/useFileManagement";
import { handleMultiFileDragStart } from "@/lib/fileUtils";
import { closeWindow } from "@/lib/windowUtils";
import { FilePreview } from "@/types";
import { DialogClose } from "@radix-ui/react-dialog";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { ChevronDown, Clipboard, Copy, Download, Settings, X } from 'lucide-react';
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { getFileExtension } from "./lib/utils";
import { StackedIcons } from "./components/StackedIcons";
import { useNavigate } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";

function App() {
  const listenerSetup = useRef(false);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const { files, addFiles, getFileIcon, clearFiles, droppedFiles } = useFileManagement();
  const navigate = useNavigate();

  useEffect(() => {
    if (listenerSetup.current) return;
    listenerSetup.current = true;

    const setupFileListener = async () => {
      const webview = await getCurrentWebview();
      await webview.onDragDropEvent(async (event) => {
        if (event.payload.type === 'drop') {
          droppedFiles();
        }
      });
    };

    setupFileListener();

    // Set up navigation event listener
    const unlisten = listen<string>("navigate_to", (event) => {
      if (event.payload) {
        navigate(event.payload);
      }
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [addFiles, getFileIcon, navigate]);

  const handleDragEnter = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    console.log("handleDragOver")
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback(async (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();

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

  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsModalOpen(true);
  }, []);

  const openSettings = () => {
    invoke('open_settings_window').catch((err) => console.error(err));
  };

  return (
    <div 
      className="fixed inset-0 text-foreground flex flex-col bg-background p-2"
      onContextMenu={handleContextMenu}
    >
      {/* Handle and Title Bar */}
      <div className="relative flex justify-end items-center h-5" data-tauri-drag-region onDragStart={(e)=>{
        console.log("drag start")
        e.preventDefault();
      }}>
        <div className="absolute left-1/2 transform -translate-x-1/2 top-1/2 -translate-y-1/2">
          <div className="w-10 h-0.5 bg-foreground rounded-full" data-tauri-drag-region></div>
        </div>
        <Button 
          variant="ghost" 
          size="icon" 
          className="text-foreground hover:bg-gray-500 hover:text-background rounded h-5 w-5 mr-1" 
          onClick={openSettings}
        >
          <Settings className="h-4 w-4" />
        </Button>
        <Button variant="ghost" size="icon" className="text-foreground hover:bg-red-500 hover:text-background rounded h-5 w-5" onClick={closeWindow}>
          <X className="h-4 w-4" />
        </Button>
      </div>


        <>
          {/* Main Content */}
          <div className="flex-grow flex flex-col items-center justify-center space-y-1"
               onDragEnter={handleDragEnter}
               onDragOver={handleDragOver}
               onDragLeave={handleDragLeave}
               onDrop={handleDrop}>
            {files.length > 0 ? (
              <div className="relative w-10 h-10 flex items-center justify-center" draggable onDragStart={handleStackDragStart}>
                <StackedIcons files={files} handleStackDragStart={handleStackDragStart} />
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
              variant="secondary"
            >
              <span>{files.length} file{files.length !== 1 ? 's' : ''}</span>
              <ChevronDown className="h-2 w-2 ml-1" />
            </Button>
          </div>
        </>

      <Dialog open={isModalOpen} onOpenChange={setIsModalOpen}>
        <DialogContent 
          className="rounded-md p-0 mt-2 w-[90vw]"
        >
          <div className="flex flex-col items-start text-foreground">
            {files.length > 0 ? (
              <>
                <Button 
                  className="w-full text-left justify-start"
                  variant="ghost"
                >
                  <Copy className="h-4 w-4 mr-2" />
                  Copy
                </Button>
                <Button 
                  className="w-full text-left justify-start"
                  variant="ghost"
                >
                  <Clipboard className="h-4 w-4 mr-2 " />
                  Paste
                </Button>
                <div className="w-[90%] h-[1px] bg-foreground mx-[5%]"></div>
                <Button 
                  className="w-full text-left justify-start hover:bg-secondary transition-colors"
                  variant="ghost"
                  onClick={() => {
                    clearFiles(files.map(file => file.id));
                  }}
                >
                  <X  className="h-4 w-4 mr-2" />
                  Clear
                </Button>
              </>
            ) : (
              <Button 
                className="w-full text-left justify-start  hover:bg-secondary transition-colors"
                variant="ghost"
              >
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