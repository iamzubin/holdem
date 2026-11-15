import { FileThumb } from '@/components/FileThumb';
import { Button } from '@/components/ui/button';
import { useFileManagement } from '@/hooks/useFileManagement';
import { useNativeDrag } from '@/hooks/useNativeDrag';
import { cn, formatFileSize } from '@/lib/utils';
import { closeCurrentWindow } from '@/lib/windowUtils';
import { FilePreview, isFolder } from '@/types';
import * as ContextMenu from '@radix-ui/react-context-menu';
import { invoke } from '@tauri-apps/api/core';
import { Grid as GridIcon, List as ListIcon } from 'lucide-react';
import React, { useCallback, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import SimpleBar from 'simplebar-react';
import 'simplebar-react/dist/simplebar.min.css';

type ViewMode = 'list' | 'grid';

const PopupWindow: React.FC = () => {
  const { t, i18n } = useTranslation();
  const { files } = useFileManagement();
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [lastSelectedFile, setLastSelectedFile] = useState<string | null>(null);
  const fileRefs = useRef<{ [key: string]: HTMLDivElement | null }>({});
  const { beginDrag, handleMouseUp } = useNativeDrag();

  // The popup opens unfocused. It closes 3s after the pointer is outside it,
  // unless the user has clicked into it; after that, it closes on blur.
  // Hovering or scrolling without clicking must not count as "unused".
  useEffect(() => {
    const root = document.documentElement;
    let timeoutId: number | undefined;
    const startCloseTimer = () => {
      window.clearTimeout(timeoutId);
      timeoutId = window.setTimeout(closeCurrentWindow, 3000);
    };
    const stopCloseTimer = () => window.clearTimeout(timeoutId);
    const handleFocus = () => {
      root.removeEventListener('mouseleave', startCloseTimer);
      stopCloseTimer();
    };
    const handleBlur = () => closeCurrentWindow();

    startCloseTimer();
    root.addEventListener('mouseenter', stopCloseTimer);
    root.addEventListener('mouseleave', startCloseTimer);
    window.addEventListener('focus', handleFocus);
    window.addEventListener('blur', handleBlur);

    return () => {
      root.removeEventListener('mouseenter', stopCloseTimer);
      root.removeEventListener('mouseleave', startCloseTimer);
      window.removeEventListener('focus', handleFocus);
      window.removeEventListener('blur', handleBlur);
      stopCloseTimer();
    };
  }, []);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent<HTMLDivElement>, file: FilePreview) => {
      if (e.button !== 0) return;
      e.stopPropagation();
      // Drag the selection only when grabbing one of its rows; grabbing any
      // other row drags just that file, as in Explorer.
      const filesToDrag = selectedFiles.has(file.id.toString())
        ? files.filter((f) => selectedFiles.has(f.id.toString()))
        : [file];
      beginDrag(filesToDrag, fileRefs.current[file.id]);
    },
    [files, selectedFiles, beginDrag],
  );

  const handleFileClick = useCallback(
    (fileId: string, event: Pick<React.MouseEvent, 'shiftKey' | 'ctrlKey' | 'metaKey'>) => {
      setSelectedFiles((prev) => {
        const newSet = new Set(prev);
        if (event.shiftKey && lastSelectedFile) {
          const fileIds = files.map((f) => f.id.toString());
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
    },
    [files, lastSelectedFile],
  );

  const locale = i18n.resolvedLanguage;
  const totalSize = formatFileSize(
    files.reduce((acc, file) => acc + file.size, 0),
    locale,
  );
  const sizeLabel = (file: FilePreview) =>
    isFolder(file) ? t('popup.folder') : formatFileSize(file.size, locale);

  // Right-clicking a row outside the selection acts on that row, as in Explorer.
  const handleRowContextMenu = useCallback((fileId: string) => {
    setSelectedFiles((prev) => (prev.has(fileId) ? prev : new Set([fileId])));
    setLastSelectedFile(fileId);
  }, []);

  const focusRow = (index: number) => {
    const file = files[Math.max(0, Math.min(files.length - 1, index))];
    if (file) fileRefs.current[file.id]?.focus();
  };

  const handleRowKeyDown = (event: React.KeyboardEvent<HTMLDivElement>, index: number) => {
    const fileId = files[index].id.toString();
    const step = isList ? 1 : 2; // grid has two columns
    switch (event.key) {
      case 'ArrowDown':
        focusRow(index + step);
        break;
      case 'ArrowUp':
        focusRow(index - step);
        break;
      case 'ArrowRight':
        if (!isList) focusRow(index + 1);
        break;
      case 'ArrowLeft':
        if (!isList) focusRow(index - 1);
        break;
      case 'Home':
        focusRow(0);
        break;
      case 'End':
        focusRow(files.length - 1);
        break;
      case ' ':
      case 'Enter':
        handleFileClick(fileId, event);
        break;
      case 'a':
        if (!(event.ctrlKey || event.metaKey)) return;
        setSelectedFiles(new Set(files.map((f) => f.id.toString())));
        break;
      case 'Delete':
        if (selectedFiles.size === 0) return;
        handleRemoveSelectedFiles();
        break;
      case 'Escape':
        if (selectedFiles.size === 0) return;
        setSelectedFiles(new Set());
        break;
      default:
        return;
    }
    event.preventDefault();
  };

  const handleRemoveSelectedFiles = useCallback(() => {
    const fileIds = Array.from(selectedFiles)
      .map((id) => Number(id))
      .filter((n) => Number.isInteger(n) && n >= 0);
    invoke('remove_files', { fileIds })
      .then(() => {
        setSelectedFiles(new Set());
      })
      .catch((error) => {
        console.error('Failed to remove selected files:', error);
      });
  }, [selectedFiles]);

  // Prune selected ids that no longer exist whenever the file list refreshes.
  useEffect(() => {
    setSelectedFiles((prev) => {
      if (prev.size === 0) return prev;
      const ids = new Set(files.map((file) => file.id.toString()));
      const next = new Set([...prev].filter((id) => ids.has(id)));
      return next.size === prev.size ? prev : next;
    });
  }, [files]);

  const isList = viewMode === 'list';

  return (
    <div
      className="fixed inset-0 flex flex-col overflow-hidden bg-background p-2 rounded border border-border"
      onContextMenu={(event) => event.preventDefault()}
    >
      <div className="flex shrink-0 justify-between items-center mb-2">
        <div className="flex items-center gap-2">
          {files.length > 0 && (
            <>
              <span className="text-xs text-primary" aria-live="polite">
                {selectedFiles.size > 0
                  ? t('popup.itemsSelected', { count: selectedFiles.size })
                  : t('app.files', { count: files.length })}
              </span>
              <span className="text-xs text-primary">{totalSize}</span>
            </>
          )}
        </div>
        <div className="flex items-center gap-1" role="group" aria-label={t('popup.viewMode')}>
          <Button
            variant={isList ? 'secondary' : 'ghost'}
            size="icon"
            className="h-7 w-7 text-primary"
            onClick={() => setViewMode('list')}
            aria-pressed={isList}
            aria-label={t('popup.list')}
          >
            <ListIcon className="h-4 w-4" />
          </Button>
          <Button
            variant={!isList ? 'secondary' : 'ghost'}
            size="icon"
            className="h-7 w-7 text-primary"
            onClick={() => setViewMode('grid')}
            aria-pressed={!isList}
            aria-label={t('popup.grid')}
          >
            <GridIcon className="h-4 w-4" />
          </Button>
        </div>
      </div>
      <SimpleBar style={{ flex: 1, minHeight: 0, height: 'auto' }} className="min-h-0">
        <div className="flex min-h-0 flex-col pb-1">
          <ContextMenu.Root>
            <ContextMenu.Trigger>
              <div
                role="listbox"
                aria-multiselectable
                aria-label={t('popup.files')}
                className={isList ? 'space-y-1 pb-1' : 'grid grid-cols-2 gap-1 pb-1'}
              >
                {files.map((file, index) => (
                  <div
                    key={file.id}
                    ref={(el) => {
                      fileRefs.current[file.id] = el;
                    }}
                    role="option"
                    aria-selected={selectedFiles.has(file.id.toString())}
                    tabIndex={0}
                    className={cn(
                      'p-1 rounded cursor-grab active:cursor-grabbing outline-none focus-visible:ring-2 focus-visible:ring-ring',
                      isList ? 'flex items-center gap-2' : 'flex flex-col items-center',
                      selectedFiles.has(file.id.toString()) && 'bg-accent/50',
                    )}
                    onClick={(e) => handleFileClick(file.id.toString(), e)}
                    onContextMenu={() => handleRowContextMenu(file.id.toString())}
                    onKeyDown={(e) => handleRowKeyDown(e, index)}
                    onMouseDown={(e) => handleMouseDown(e, file)}
                    onMouseUp={handleMouseUp}
                    onMouseLeave={handleMouseUp}
                  >
                    <div
                      className={
                        isList
                          ? 'flex items-center justify-center overflow-hidden w-8 h-8 flex-shrink-0'
                          : 'flex items-center justify-center overflow-hidden w-12 h-12 mb-1'
                      }
                    >
                      <FileThumb file={file} />
                    </div>
                    <div className={isList ? 'flex-grow min-w-0' : 'w-full text-center'}>
                      <p
                        className="text-xs text-primary font-medium truncate"
                        title={file.name}
                      >
                        {file.name}
                      </p>
                      {!isList && (
                        <span className="text-[10px] text-primary">{sizeLabel(file)}</span>
                      )}
                    </div>
                    {isList && (
                      <span className="text-[10px] text-primary flex-shrink-0">
                        {sizeLabel(file)}
                      </span>
                    )}
                  </div>
                ))}
              </div>
            </ContextMenu.Trigger>
            <ContextMenu.Portal>
              <ContextMenu.Content className="min-w-[200px] bg-background rounded-md overflow-hidden p-1">
                <ContextMenu.Item
                  onClick={handleRemoveSelectedFiles}
                  disabled={selectedFiles.size === 0}
                  className="text-xs text-primary rounded px-2 py-1.5 cursor-pointer hover:bg-secondary focus:bg-secondary outline-none disabled:cursor-not-allowed disabled:opacity-40"
                >
                  {t('popup.removeSelected')}
                </ContextMenu.Item>
              </ContextMenu.Content>
            </ContextMenu.Portal>
          </ContextMenu.Root>
        </div>
      </SimpleBar>
    </div>
  );
};

export default PopupWindow;
