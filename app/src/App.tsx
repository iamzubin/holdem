"use client"

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { useFileManagement } from "@/hooks/useFileManagement";
import { closeWindow } from "@/lib/windowUtils";
import { FileWithPath } from "@/types";
import { DialogClose } from "@radix-ui/react-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ChevronDown, Download, Settings, X } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { StackedIcons } from "./components/StackedIcons";
import { useNavigate } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { useTheme } from "@/components/theme-provider";

function App() {
  const { t } = useTranslation();
  const { theme } = useTheme();
  const listenerSetup = useRef(false);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const { files, clearFiles, droppedFiles } = useFileManagement();
  const navigate = useNavigate();

  // Check analytics consent on mount
  useEffect(() => {
    const checkConsent = async () => {
      try {
        const configExists = await invoke<boolean>('check_config_exists');

        if (!configExists) {
          // Add a small delay to ensure main window is fully loaded
          setTimeout(async () => {
            await invoke('open_consent_window');
          }, 500);
        }
      } catch (error) {
        console.error('Failed to check config existence:', error);
        // If we can't check, don't show consent window to be safe
      }
    };

    checkConsent();
  }, []);

  useEffect(() => {
    if (listenerSetup.current) return;
    listenerSetup.current = true;
    let cancelled = false;
    let unlistenNavigate: (() => void) | undefined;

    // Drops are owned by the native HoldemDropTarget (see
    // src-tauri/src/drop_target.rs). The frontend only forwards blob-backed
    // virtual files that the backend can never resolve (blob: URLs are
    // opaque to reqwest by design). Everything else — real paths, HTML,
    // URLs, plain text, bitmaps, FileGroupDescriptor virtual files — is
    // handled natively and arrives via the `files_updated` event.

    // Set up navigation event listener
    const unlisten = listen<string>("navigate_to", (event) => {
      if (event.payload) {
        navigate(event.payload);
      }
    });
    unlisten.then(fn => {
      if (cancelled) {
        fn();
      } else {
        unlistenNavigate = fn;
      }
    });

    return () => {
      cancelled = true;
      unlisten.then(fn => fn());
      if (unlistenNavigate) unlistenNavigate();
      // Reset the guard so a re-run (e.g. StrictMode remount) re-registers
      // listeners instead of exiting early while they stay removed.
      listenerSetup.current = false;
    };
  }, [navigate]);

  const handleDragEnter = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    e.dataTransfer.dropEffect = 'copy';
  }, []);

  // Thin forwarder only: backend owns all drops EXCEPT blob-backed virtual
  // files (browser canvas / Discord / blob:<uuid>), which reqwest can never
  // resolve. Those arrive here as File objects WITHOUT a .path — read them
  // via FileReader and hand the bytes to the backend. Everything else
  // returns early so the same physical drop is never processed twice.
  const handleDrop = useCallback(async (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();

    const files = Array.from(e.dataTransfer.files);
    const virtualFiles = files.filter((f) => !(f as FileWithPath).path);

    if (virtualFiles.length === 0) {
      // Real files, HTML, URLs, text, bitmaps, FileGroupDescriptor drops:
      // the native drop target already shelved them. Just refresh.
      droppedFiles();
      return;
    }

    invoke('mark_drop_received').catch(() => {});
    for (const file of virtualFiles) {
      const reader = new FileReader();
      const done = new Promise<void>((resolve) => {
        reader.onload = async () => {
          try {
            const result = reader.result as string;
            const base64Data = result.includes(',') ? result.split(',')[1] : result;
            let extension = 'png';
            if (file.name && file.name.includes('.')) {
              extension = file.name.split('.').pop()?.toLowerCase() || 'png';
            } else if (file.type) {
              extension = file.type.split('/')[1] || 'png';
            }
            const path = await invoke<string>('save_pasted_data_base64', {
              dataBase64: base64Data,
              extension,
              originalName: file.name || null,
            });
            await invoke('add_files', { files: [path] });
            droppedFiles();
          } catch (err) {
            console.error('Failed to save dropped virtual (blob) file', err);
          }
          resolve();
        };
        reader.onerror = () => {
          console.error('Failed to read dropped virtual file', reader.error);
          resolve();
        };
      });
      reader.readAsDataURL(file);
      await done;
    }
  }, [droppedFiles]);

  const openPopup = () => {
    invoke('open_popup_window', { theme }).catch((err) => console.error(err));
  };

  const stackedIconsRef = useRef<HTMLDivElement>(null);

  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsModalOpen(true);
  }, []);

  const openSettings = () => {
    invoke('open_settings_window', { theme }).catch((err) => console.error(err));
  };

  return (
    <div
      className="fixed inset-0 text-foreground flex flex-col bg-background p-2 focus:outline-none"
      onContextMenu={handleContextMenu}
      tabIndex={0}
      onDragEnter={handleDragEnter}
      onDragLeave={handleDragLeave}
      onDragOver={handleDragOver}
      onDrop={handleDrop}
    >
      {/* Handle and Title Bar */}
      <div className="relative flex justify-end items-center h-5" data-tauri-drag-region onDragStart={(e) => {
        e.preventDefault();
      }}>
        <div className="absolute left-1/2 transform -translate-x-1/2 top-1/2 -translate-y-1/2">
          <div className="w-10 h-0.5 bg-foreground rounded-full" data-tauri-drag-region></div>
        </div>
        <Button
          variant="ghost"
          size="icon"
          className="text-foreground hover:bg-gray-500 hover:text-background rounded h-5 w-5"
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
        <div className="grow flex flex-col items-center justify-center space-y-1"
          onDragEnter={handleDragEnter}
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}>
          {files.length > 0 ? (
            <div ref={stackedIconsRef} className="relative w-10 h-10 flex items-center justify-center">
              <StackedIcons files={files} />
            </div>
          ) : (
            <div className="flex flex-col items-center">
              <Download className="h-5 w-5" />
              <span className="text-[8px]">{t("app.dropHere")}</span>
            </div>
          )}
        </div>

        {/* Dropdown Button at the Bottom */}
        <div className="flex justify-center items-center mt-1">
          <Button
            onClick={openPopup}
            variant="secondary"
          >
            <span>{t("app.files", { count: files.length })}</span>
            <ChevronDown className="h-2 w-2 ms-1" />
          </Button>
        </div>
      </>

      <Dialog open={isModalOpen} onOpenChange={setIsModalOpen}>
        <DialogContent
          className="rounded-md p-0 mt-2 w-[90vw]"
          aria-describedby={undefined}
        >
          <DialogTitle className="sr-only">{t("app.contextMenuTitle")}</DialogTitle>
          <div className="flex flex-col items-start text-foreground">
            {files.length > 0 && (
              <Button
                className="w-full text-left justify-start hover:bg-secondary transition-colors"
                variant="ghost"
                onClick={() => {
                  clearFiles(files.map(file => file.id));
                  setIsModalOpen(false);
                }}
              >
                <X className="h-4 w-4 me-2" />
                {t("app.clear")}
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
