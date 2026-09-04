import { DynamicFileIcon } from "@/components/FileIcon";
import { useTheme } from "@/components/theme-provider";
import { Button } from "@/components/ui/button";
import { useFileManagement } from "@/hooks/useFileManagement";
import { setPendingFiles, prepareDragImage, triggerNativeDrag } from "@/lib/fileUtils";
import { POPUP_SELECT_ALL_EVENT } from "./ContextMenuWindow";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { List as ListIcon, Grid as GridIcon } from 'lucide-react';
import React, { useEffect, useState, useCallback, useRef } from "react";
import { Toaster } from "sonner";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import SimpleBar from 'simplebar-react';
import 'simplebar-react/dist/simplebar.min.css';
import { useTranslation } from 'react-i18next';

const PopupWindow: React.FC = () => {
  const { t } = useTranslation();
  const { theme } = useTheme();
  const { files } = useFileManagement();
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [hasInteracted, setHasInteracted] = useState(false);
  const [viewMode, setViewMode] = useState<'list' | 'grid'>('list');
  const [lastSelectedFile, setLastSelectedFile] = useState<string | null>(null);
  const fileRefs = useRef<{ [key: string]: HTMLDivElement | null }>({});
  const dragTimeoutRef = useRef<number | null>(null);

  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (!hasInteracted) {
        invoke('close_popup_window').catch((error) => {
          console.error('Failed to close popup window after inactivity:', error);
        });
      }
    }, 3000);

    const handleFocus = () => {
      clearTimeout(timeoutId);
      setHasInteracted(true);
    };

    const handleBlur = () => {
      if (hasInteracted) {
        invoke('close_popup_window').catch((error) => {
          console.error('Failed to close popup window on blur:', error);
        });
      }
    };

    window.addEventListener('focus', handleFocus);
    window.addEventListener('blur', handleBlur);

    return () => {
      window.removeEventListener('focus', handleFocus);
      window.removeEventListener('blur', handleBlur);
      clearTimeout(timeoutId);
    };
  }, [hasInteracted]);

  const isMouseDownRef = useRef(false);

  const handleMouseDown = useCallback((e: React.MouseEvent<HTMLDivElement>, file: any) => {
    // Only trigger on left mouse button
    if (e.button !== 0) return;
    
    e.stopPropagation();
    isMouseDownRef.current = true;
    
    // Determine which files to drag
    let filesToDrag: any[];
    if (selectedFiles.size > 0) {
      filesToDrag = files.filter(f => selectedFiles.has(f.id.toString()));
    } else {
      filesToDrag = [file];
    }
    
    setPendingFiles(filesToDrag);
    
    // Get the element for image capture
    const element = fileRefs.current[file.id];
    if (element) {
      prepareDragImage(element).then(() => {
        if (isMouseDownRef.current) {
          dragTimeoutRef.current = window.setTimeout(() => {
            if (isMouseDownRef.current) {
              triggerNativeDrag();
            }
          }, 150);
        }
      });
    } else {
      dragTimeoutRef.current = window.setTimeout(() => {
        if (isMouseDownRef.current) {
          triggerNativeDrag();
        }
      }, 150);
    }
  }, [files, selectedFiles]);

  const handleMouseUp = useCallback(() => {
    isMouseDownRef.current = false;
    if (dragTimeoutRef.current) {
      clearTimeout(dragTimeoutRef.current);
      dragTimeoutRef.current = null;
    }
  }, []);

  const handleFileClick = useCallback((fileId: string, event: React.MouseEvent) => {
    setSelectedFiles(prev => {
      const newSet = new Set(prev);
      if (event.shiftKey && lastSelectedFile) {
        const fileIds = files.map(f => f.id.toString());
        const startIndex = fileIds.indexOf(lastSelectedFile);
        const endIndex = fileIds.indexOf(fileId);
        const [start, end] = [Math.min(startIndex, endIndex), Math.max(startIndex, endIndex)];
        for (let i = start; i <= end; i++) {
          newSet.add(fileIds[i]);
        }
      } else if (event.ctrlKey || event.metaKey) {
        if (newSet.has(fileId)) {
          newSet.delete(fileId);
        } else {
          newSet.add(fileId);
        }
      } else {
        newSet.clear();
        newSet.add(fileId);
      }
      return newSet;
    });
    setLastSelectedFile(fileId);
  }, [files, lastSelectedFile]);

  const getTotalSize = (files: any[]): string => {
    const totalBytes = files.reduce((acc, file) => acc + file.size, 0);
    return formatFileSize(totalBytes);
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024 * 1024) {
      return `${(bytes / 1024).toFixed(1)} KB`;
    }
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const toggleViewMode = () => {
    setViewMode(prev => prev === 'list' ? 'grid' : 'list');
  };

  // The context menu lives in its own non-activating window (see
  // ContextMenuWindow). Any mousedown here dismisses it; a right-click
  // reopens it at the cursor via the backend.
  const dismissMenu = useCallback(() => {
    invoke('close_context_menu_window').catch(() => {});
  }, []);

  const handleContextMenu = useCallback((event: React.MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
    invoke('open_context_menu_window', {
      theme,
      fileIds: Array.from(selectedFiles).map((id) => parseInt(id)),
    }).catch((error) => {
      console.error('Failed to open context menu window:', error);
    });
  }, [theme, selectedFiles]);

  // Removal now happens from the context menu window, so prune ids that
  // no longer exist whenever the file list refreshes.
  useEffect(() => {
    setSelectedFiles((prev) => {
      if (prev.size === 0) return prev;
      const ids = new Set(files.map((file) => file.id.toString()));
      const next = new Set([...prev].filter((id) => ids.has(id)));
      return next.size === prev.size ? prev : next;
    });
  }, [files]);

  // "Select All" picked in the context menu window.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    listen(POPUP_SELECT_ALL_EVENT, () => {
      setSelectedFiles(new Set(files.map((file) => file.id.toString())));
    })
      .then((fn) => {
        if (cancelled) fn();
        else unlisten = fn;
      })
      .catch(() => {});
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [files]);

  return (
    <div
      className="fixed inset-0 flex flex-col overflow-hidden bg-background p-2 rounded border border-border"
      onContextMenu={handleContextMenu}
      onMouseDownCapture={dismissMenu}
    >
      <div className="flex shrink-0 justify-between items-center mb-2">
        <div className="flex items-center gap-2">
          {files.length > 0 && (
            <>
              <span className="text-xs text-primary">{t("popup.itemsSelected", { count: files.length })}</span>
              <span className="text-xs text-primary">{getTotalSize(files)}</span>
            </>
          )}
        </div>
        <div className="flex gap-2">
          <ToggleGroup
            type="single"
            value={viewMode}
            onValueChange={toggleViewMode}
            >
            <ToggleGroupItem value="list" className="text-primary">
              <span className="sr-only">{t("popup.list")}</span>
              <ListIcon className="h-4 w-4" />
            </ToggleGroupItem>
            <ToggleGroupItem value="grid" className="text-primary">
              <span className="sr-only">{t("popup.grid")}</span>
              <GridIcon className="h-4 w-4" />
            </ToggleGroupItem>
          </ToggleGroup>
        </div>
      </div>
      <SimpleBar id="RSC-Example" style={{ flex: 1, minHeight: 0, height: 'auto' }} className="min-h-0">
      <div className="flex min-h-0 flex-col pb-1">
          <div className={`${viewMode === 'grid' ? 'grid grid-cols-2 gap-1' : 'space-y-1'} pb-1`}>
            {files.map(file => (
              <div
                key={file.id}
                ref={el => { fileRefs.current[file.id] = el; }}
                className={`
                  ${viewMode === 'list'
                    ? 'flex items-center gap-2 p-1 rounded'
                    : 'flex flex-col items-center p-1 rounded'
                  }
                  ${selectedFiles.has(file.id.toString()) ? 'bg-accent bg-opacity-50' : ''}
                  cursor-grab active:cursor-grabbing
                `}
                onClick={(e) => handleFileClick(file.id.toString(), e)}
                onMouseDown={(e) => handleMouseDown(e, file)}
                onMouseUp={handleMouseUp}
                onMouseLeave={handleMouseUp}
              >
                <div className={`
                   flex items-center justify-center overflow-hidden
                  ${viewMode === 'list' ? 'w-8 h-8 flex-shrink-0' : 'w-12 h-12 mb-1'}
                `}>
                  {file.preview ? (
                    <img src={file.preview} alt={file.name} className="w-full h-full object-cover" />
                  ) : (
                    <DynamicFileIcon file={file} />
                  )}
                </div>
                <div className={`
                  ${viewMode === 'list' ? 'flex-grow min-w-0' : 'w-full text-center'}
                `}>
                  <p className="text-xs text-primary font-medium truncate" title={file.name}>{file.name}</p>
                  {viewMode === 'grid' && (
                    <span className="text-[10px] text-primary">{formatFileSize(file.size)}</span>
                  )}
                </div>
                {viewMode === 'list' && (
                  <span className="text-[10px] text-primary flex-shrink-0">{formatFileSize(file.size)}</span>
                )}
              </div>
            ))}
          </div>
      </div>
      </SimpleBar>

      <Toaster />
    </div>
  );
};

export default PopupWindow;
