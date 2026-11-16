import { FileThumb } from '@/components/FileThumb';
import { PreviewPane } from '@/components/PreviewPane';
import { ShelfTabs } from '@/components/ShelfTabs';
import { createShelf, cycleShelf, reopenShelf, shelfName } from '@/lib/shelves';
import { Button } from '@/components/ui/button';
import { MENU_ITEM } from '@/components/ui/menu';
import { useFileManagement } from '@/hooks/useFileManagement';
import { useNativeDrag } from '@/hooks/useNativeDrag';
import { useTauriEvent } from '@/hooks/useTauriEvent';
import { ShelfNotice, shelfNoticeText } from '@/lib/notices';
import { cn, formatFileSize } from '@/lib/utils';
import { FilePreview, isConvertibleImage, isFolder } from '@/types';
import * as ContextMenu from '@radix-ui/react-context-menu';
import { invoke } from '@tauri-apps/api/core';
import { ChevronRight, CloudOff, Grid as GridIcon, List as ListIcon, Pin } from 'lucide-react';
import React, { useCallback, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import SimpleBar from 'simplebar-react';
import 'simplebar-react/dist/simplebar.min.css';

type ViewMode = 'list' | 'grid';

const NOTICE_MS = 6000;
/** Failures stay longer: they explain what to do next. */
const ERROR_NOTICE_MS = 12000;
const GRID_COLUMNS = 3;


/** Keyboard shortcut shown at the end of a menu item. */
const Hint: React.FC<{ keys: string }> = ({ keys }) => (
  <span className="text-[11px] text-muted-foreground">{keys}</span>
);

const closePopup = () =>
  invoke('close_popup').catch((error) => console.error('Failed to hide the list:', error));

/** A file name without its extension: what Rename edits. */
const stemOf = (name: string) => {
  const dot = name.lastIndexOf('.');
  return dot > 0 ? name.slice(0, dot) : name;
};

type ImageResize =
  | { kind: 'none' }
  | { kind: 'percent'; value: number }
  | { kind: 'fit'; value: number };
const RESIZES: Extract<ImageResize, { value: number }>[] = [
  { kind: 'percent', value: 50 },
  { kind: 'percent', value: 25 },
  { kind: 'fit', value: 1920 },
  { kind: 'fit', value: 1280 },
];

const PopupWindow: React.FC = () => {
  const { t, i18n } = useTranslation();
  const { files, allFiles, shelves, activeShelf, closedShelves } = useFileManagement();
  const shelfCounts = new Map<number, number>();
  for (const file of allFiles) shelfCounts.set(file.shelf, (shelfCounts.get(file.shelf) ?? 0) + 1);
  // Space-bar preview of one item.
  const [previewId, setPreviewId] = useState<number | null>(null);
  const previewIndex = files.findIndex((file) => file.id === previewId);
  const previewFile = previewIndex >= 0 ? files[previewIndex] : undefined;
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [lastSelectedFile, setLastSelectedFile] = useState<string | null>(null);
  const fileRefs = useRef<{ [key: string]: HTMLDivElement | null }>({});
  // The item menu is open, or closed during the current key press.
  const menuOpen = useRef(false);
  // Roving tabindex: the list is a single Tab stop; arrows move within it.
  const [activeIndex, setActiveIndex] = useState(0);
  const { beginDrag } = useNativeDrag();
  const [isDropHover, setIsDropHover] = useState(false);
  // Message bar under the list, with Undo after a removal or a closed shelf.
  const [notice, setNotice] = useState<{ text: string; onUndo?: () => void } | null>(null);
  const noticeTimer = useRef<number | undefined>(undefined);
  const noticeMs = useRef(NOTICE_MS);
  const startNoticeTimer = useCallback(() => {
    window.clearTimeout(noticeTimer.current);
    if (noticeMs.current > 0) {
      noticeTimer.current = window.setTimeout(() => setNotice(null), noticeMs.current);
    }
  }, []);
  /** Shows a message under the list. It stays while pointed at or focused;
   * `persist` keeps it until the next one (work in progress). */
  const showNotice = useCallback(
    (text: string, options: { onUndo?: () => void; error?: boolean; persist?: boolean } = {}) => {
      setNotice({ text, onUndo: options.onUndo });
      noticeMs.current = options.persist ? 0 : options.error ? ERROR_NOTICE_MS : NOTICE_MS;
      startNoticeTimer();
    },
    [startNoticeTimer],
  );
  // The item being renamed in place, and the name typed so far. The ref
  // lets the blur that follows Enter or Esc see that the edit is over.
  const [renaming, setRenaming] = useState<{ id: number; draft: string } | null>(null);
  const renamingRef = useRef(renaming);
  renamingRef.current = renaming;
  // First open from the keyboard: the page wasn't listening for
  // `popup-shown` yet, so the window URL says so instead.
  const keyboardOpen = useRef(new URLSearchParams(window.location.search).has('keyboard'));
  useEffect(() => () => window.clearTimeout(noticeTimer.current), []);

  // Drops on the list that couldn't be kept, and items that went missing.
  useTauriEvent<ShelfNotice>('shelf-notice', (event) => {
    const { full, error } = shelfNoticeText(event, t);
    showNotice(full, { error });
  });

  // The backend shows and hides this window (it is never closed) and
  // decides when it is no longer in use (window_ops.rs). Opened from the
  // keyboard, focus goes to the first item.
  useTauriEvent<boolean>('popup-shown', (keyboard) => {
    if (!keyboard || files.length === 0) return;
    setActiveIndex(0);
    fileRefs.current[files[0].id]?.focus();
  });
  // Hidden: start fresh next time. A rename still being typed is dropped.
  useTauriEvent('popup-hidden', () => {
    renamingRef.current = null;
    setSelectedFiles(new Set());
    setPreviewId(null);
    setNotice(null);
    setRenaming(null);
  });
  // Files dragged over the list: the page gets no drag events itself, so the
  // native drop target reports them.
  useTauriEvent<boolean>('drop-hover', setIsDropHover);
  // Should a drop ever reach the page instead (WebView2 recreating its
  // window before Holdem's target is registered again), refuse it rather
  // than let the browser open the file in place of the list.
  useEffect(() => {
    const refuse = (event: DragEvent) => event.preventDefault();
    window.addEventListener('dragover', refuse);
    window.addEventListener('drop', refuse);
    return () => {
      window.removeEventListener('dragover', refuse);
      window.removeEventListener('drop', refuse);
    };
  }, []);

  const handlePointerDown = useCallback(
    (e: React.PointerEvent<HTMLDivElement>, file: FilePreview) => {
      if (e.button !== 0) return;
      e.stopPropagation();
      // Drag the selection only when grabbing one of its rows; grabbing any
      // other row drags just that file, as in Explorer. Offline items can't
      // be dragged anywhere, so they are left out.
      const filesToDrag = (
        selectedFiles.has(file.id.toString())
          ? files.filter((f) => selectedFiles.has(f.id.toString()))
          : [file]
      ).filter((f) => f.available);
      if (filesToDrag.length > 0) beginDrag(filesToDrag, fileRefs.current[file.id], e);
    },
    [files, selectedFiles, beginDrag],
  );

  const handleFileClick = useCallback(
    (fileId: string, event: Pick<React.MouseEvent, 'shiftKey' | 'ctrlKey' | 'metaKey'>) => {
      setSelectedFiles((prev) => {
        const newSet = new Set(prev);
        const fileIds = files.map((f) => f.id.toString());
        // The anchor may have left the list (removed, other shelf): then
        // Shift+click selects just this item.
        const startIndex = lastSelectedFile ? fileIds.indexOf(lastSelectedFile) : -1;
        if (event.shiftKey && startIndex >= 0) {
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
  // With a selection, the size shown is the selection's, not the shelf's.
  const counted =
    selectedFiles.size > 0 ? files.filter((f) => selectedFiles.has(f.id.toString())) : files;
  const totalBytes = counted.reduce((acc, file) => acc + file.size, 0);
  // Folder sizes are not computed, so a shelf of folders has no meaningful total.
  const totalSize = totalBytes > 0 ? formatFileSize(totalBytes, locale) : null;
  const sizeLabel = (file: FilePreview) =>
    !file.available
      ? t('popup.offline')
      : isFolder(file)
        ? t('popup.folder')
        : formatFileSize(file.size, locale);

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
    if (event.target !== event.currentTarget) return;
    const fileId = files[index].id.toString();
    const step = isList ? 1 : GRID_COLUMNS;
    // In right-to-left languages the grid's columns are mirrored.
    const forward = document.documentElement.dir === 'rtl' ? 'ArrowLeft' : 'ArrowRight';
    switch (event.key) {
      case 'ArrowDown':
        focusRow(index + step);
        break;
      case 'ArrowUp':
        focusRow(index - step);
        break;
      case 'ArrowRight':
      case 'ArrowLeft':
        if (!isList) focusRow(event.key === forward ? index + 1 : index - 1);
        break;
      case 'Home':
        focusRow(0);
        break;
      case 'End':
        focusRow(files.length - 1);
        break;
      case ' ':
        // Ctrl+Space selects, as in Explorer; Space alone previews.
        if (event.ctrlKey || event.metaKey) handleFileClick(fileId, event);
        else setPreviewId(files[index].id);
        break;
      case 'Enter':
        openFile(files[index]);
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
    invoke<number>('remove_files', { fileIds })
      .then((count) => {
        setSelectedFiles(new Set());
        if (count > 0) showNotice(t('app.removed', { count }), { onUndo: undoRemove });
      })
      .catch((error) => {
        console.error('Failed to remove selected files:', error);
        showNotice(t('popup.actionFailed'));
      });
  }, [selectedFiles, showNotice, t]);

  /** Renames the item being edited, if the name changed (owned items only). */
  const finishRename = (save: boolean) => {
    const current = renamingRef.current;
    if (!current) return;
    renamingRef.current = null;
    setRenaming(null);
    const file = files.find((f) => f.id === current.id);
    fileRefs.current[current.id]?.focus();
    const stem = current.draft.trim();
    if (!save || !file || stem === '' || stem === stemOf(file.name)) return;
    invoke('rename_item', { id: current.id, name: stem }).catch((error: unknown) => {
      console.error('Rename failed:', error);
      const reason = error === 'exists' || error === 'inUse' ? error : 'failed';
      const message = {
        exists: 'popup.renameExists',
        inUse: 'popup.renameInUse',
        failed: 'popup.renameFailed',
      }[reason];
      showNotice(t(message), { error: true });
    });
  };

  const startRename = (file: FilePreview | undefined) => {
    if (!file?.available) return;
    if (!file.owned) {
      showNotice(t('popup.renameOwnFile'));
      return;
    }
    setRenaming({ id: file.id, draft: stemOf(file.name) });
  };

  /** Converts and/or resizes the selected images; results land on this shelf. */
  const transformImages = (format: 'same' | 'png' | 'jpeg', resize: ImageResize) => {
    // Files already in the target format are left out of a plain convert.
    const already = (file: FilePreview) => {
      const type = file.file_type.toLowerCase();
      return format === 'png' ? type === 'png' : format === 'jpeg' && ['jpg', 'jpeg'].includes(type);
    };
    const ids = selectedList()
      .filter(isConvertibleImage)
      .filter((file) => resize.kind !== 'none' || !already(file))
      .map((file) => file.id);
    if (ids.length === 0) return;
    showNotice(t('image.working'), { persist: true });
    invoke<{ added: number; failed: number; animated: number }>('transform_images', {
      ids,
      format,
      resize,
    })
      .then(({ added, failed, animated }) => {
        const parts = [
          added > 0 && t('image.done', { count: added }),
          failed > 0 && t('image.failed', { count: failed }),
          animated > 0 && t('image.animated', { count: animated }),
        ].filter(Boolean);
        showNotice(parts.join('. '), { error: failed > 0 || animated > 0 });
      })
      .catch((error) => {
        console.error('Image conversion failed:', error);
        showNotice(t('image.failed', { count: ids.length }), { error: true });
      });
  };

  function selectedList() {
    return files.filter((f) => selectedFiles.has(f.id.toString()));
  }
  function undoRemove() {
    setNotice(null);
    invoke('restore_removed').catch((error) => console.error('Failed to undo:', error));
  }
  const firstSelected = selectedFiles.size === 1 ? selectedList()[0] : undefined;
  const anySelectedAvailable = selectedList().some((file) => file.available);
  const allSelectedPinned =
    selectedFiles.size > 0 && selectedList().every((file) => file.pinned);
  const selectedImages = selectedList().filter(isConvertibleImage);

  const openFile = (file: FilePreview) => {
    if (!file.available) {
      showNotice(t('popup.offlineHint'));
      return;
    }
    invoke('open_shelf_item', { id: file.id }).catch((error) => {
      console.error('Failed to open file:', error);
      showNotice(t('popup.actionFailed'));
    });
  };

  const revealFile = (id: number) => {
    invoke('reveal_shelf_item', { id }).catch((error) => {
      console.error('Failed to show file in folder:', error);
      showNotice(t('popup.actionFailed'));
    });
  };

  const copyPaths = () => {
    const paths = selectedList().map((f) => f.path);
    navigator.clipboard
      .writeText(paths.join('\r\n'))
      .then(() => showNotice(t('popup.copied', { count: paths.length })))
      .catch(() => showNotice(t('popup.actionFailed')));
  };

  // Ctrl+C / Copy: the selected items go on the clipboard as files, so
  // Ctrl+V in Explorer, mail or chat pastes them (keyboard alternative to
  // dragging).
  const copySelected = useCallback(() => {
    const ids = files.filter((f) => selectedFiles.has(f.id.toString())).map((f) => f.id);
    if (ids.length === 0) return;
    invoke<number>('copy_items_to_clipboard', { ids })
      .then((count) =>
        // Nothing copied: everything selected is offline.
        showNotice(count > 0 ? t('popup.copiedFiles', { count }) : t('popup.offlineHint')),
      )
      .catch((error) => {
        console.error('Copy failed:', error);
        showNotice(t('popup.actionFailed'));
      });
  }, [files, selectedFiles, showNotice, t]);

  // Shortcuts work wherever focus is in the popup, not only on a row:
  // Ctrl+V adds the clipboard, Ctrl+A selects all, Ctrl+C copies, Delete
  // removes, Esc clears the selection and then closes.
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      // An open menu handles its own keys (also the Esc that just closed it).
      if (menuOpen.current || document.querySelector('[role=menu]')) return;
      const ctrl = event.ctrlKey || event.metaKey;
      const key = event.key.toLowerCase();
      const onTab = (event.target as HTMLElement | null)?.closest('[role=tab]') !== null;
      if (previewFile) {
        // While previewing: arrows step through the items, Space/Esc close.
        const step =
          event.key === 'ArrowDown' || event.key === 'ArrowRight'
            ? 1
            : event.key === 'ArrowUp' || event.key === 'ArrowLeft'
              ? -1
              : 0;
        if (step !== 0) {
          const next = files[(previewIndex + step + files.length) % files.length];
          setPreviewId(next.id);
          setSelectedFiles(new Set([next.id.toString()]));
        } else if (event.key === ' ' || event.key === 'Escape') {
          setPreviewId(null);
          fileRefs.current[previewFile.id]?.focus();
        } else {
          return;
        }
        event.preventDefault();
        return;
      }
      if (ctrl && event.key === 'Tab') {
        cycleShelf(shelves, activeShelf, event.shiftKey ? -1 : 1);
      } else if (ctrl && event.shiftKey && key === 't') {
        if (closedShelves.length > 0) reopenShelf();
      } else if (event.key === 'F2' && !onTab) {
        const focused = document.activeElement?.getAttribute('role') === 'option';
        startRename(firstSelected ?? (focused ? files[activeIndex] : undefined));
      } else if (ctrl && key === 'v') {
        invoke('paste_clipboard').catch((error) => console.error('Paste failed:', error));
      } else if (ctrl && key === 'a') {
        setSelectedFiles(new Set(files.map((f) => f.id.toString())));
      } else if (ctrl && key === 'c') {
        copySelected();
      } else if (event.key === 'Delete' && selectedFiles.size > 0 && !onTab) {
        handleRemoveSelectedFiles();
      } else if (event.key === 'Escape') {
        if (selectedFiles.size > 0) setSelectedFiles(new Set());
        else closePopup();
      } else {
        return;
      }
      event.preventDefault();
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [files, selectedFiles, copySelected, handleRemoveSelectedFiles, previewFile, previewIndex, shelves, activeShelf, firstSelected, activeIndex, closedShelves]);

  const selectedIds = () => selectedList().map((file) => file.id);

  /** Puts the selected items in one ZIP archive, which lands on this shelf. */
  const compressSelected = () => {
    const ids = selectedIds();
    if (ids.length === 0) return;
    showNotice(t('actions.compressing'));
    invoke<string>('compress_items', { ids, archiveName: t('actions.archiveName') })
      .then((name) => showNotice(t('actions.compressed', { name })))
      .catch((error) => {
        console.error('Compress failed:', error);
        showNotice(t('actions.compressFailed'));
      });
  };

  /** Asks for a folder and copies the selected items there. */
  const copySelectedToFolder = () => {
    const ids = selectedIds();
    if (ids.length === 0) return;
    invoke<{ folder: string | null; count: number; complete: boolean }>('copy_items_to_folder', {
      ids,
      title: t('actions.pickFolder'),
    })
      .then(({ folder, count, complete }) => {
        if (folder === null) return;
        showNotice(
          complete
            ? t('actions.copiedTo', { count, folder })
            : t('actions.copyIncomplete', { folder }),
        );
      })
      .catch((error) => {
        console.error('Copy to folder failed:', error);
        showNotice(t('popup.actionFailed'));
      });
  };

  const moveSelected = (shelf: number) => {
    const ids = selectedIds();
    invoke<number>('move_to_shelf', { ids, shelf })
      .then((count) => {
        const target = shelves.find((s) => s.id === shelf);
        if (count > 0 && target) {
          showNotice(t('shelves.moved', { count, name: shelfName(target, t) }));
        }
      })
      .catch((error) => console.error('Move failed:', error));
  };

  const moveSelectedToNewShelf = () => {
    const ids = selectedIds();
    // Creating it makes it the active shelf; the items follow.
    createShelf(shelves, t)
      .then((shelf) => invoke('move_to_shelf', { ids, shelf }))
      .catch((error) => console.error('Move failed:', error));
  };

  const setPinned = (pinned: boolean) => {
    const ids = files.filter((f) => selectedFiles.has(f.id.toString())).map((f) => f.id);
    invoke('set_pinned', { ids, pinned }).catch((error) => console.error('Pin failed:', error));
  };

  useEffect(() => {
    setSelectedFiles(new Set());
    setPreviewId(null);
  }, [activeShelf]);

  // Prune selected ids that no longer exist whenever the file list refreshes.
  useEffect(() => {
    setSelectedFiles((prev) => {
      if (prev.size === 0) return prev;
      const ids = new Set(files.map((file) => file.id.toString()));
      const next = new Set([...prev].filter((id) => ids.has(id)));
      return next.size === prev.size ? prev : next;
    });
  }, [files]);

  // Keep the Tab stop on an existing row as files come and go.
  useEffect(() => {
    if (activeIndex >= files.length && files.length > 0) setActiveIndex(files.length - 1);
    if (keyboardOpen.current && files.length > 0) {
      keyboardOpen.current = false;
      fileRefs.current[files[0].id]?.focus();
    }
  }, [files, activeIndex]);

  const isList = viewMode === 'list';

  return (
    <div
      className={cn(
        'fixed inset-0 flex flex-col overflow-hidden bg-background p-2 transition-colors',
        isDropHover && 'ring-2 ring-inset ring-primary',
      )}
      onContextMenu={(event) => event.preventDefault()}
    >
      {/* Tint while files are dragged over. An overlay, not a background
          swap: the window is transparent (Mica), so the page background
          must stay solid. */}
      {isDropHover && (
        <div aria-hidden className="pointer-events-none absolute inset-0 z-10 bg-primary/10" />
      )}
      <div className="mb-2 flex shrink-0 items-center justify-between gap-2">
        <ShelfTabs
          shelves={shelves}
          active={activeShelf}
          counts={shelfCounts}
          closed={closedShelves}
          onDeleted={(id, count) =>
            showNotice(
              count > 0 ? t('shelves.deleted', { count }) : t('shelves.deletedEmpty'),
              { onUndo: () => reopenShelf(id).then(() => setNotice(null)) },
            )
          }
        />
        <div className="flex shrink-0 items-center gap-1" role="group" aria-label={t('popup.viewMode')}>
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
          <ContextMenu.Root
            onOpenChange={(open) => {
              if (open) menuOpen.current = true;
              else window.setTimeout(() => (menuOpen.current = false), 0);
            }}
          >
            <ContextMenu.Trigger>
              <div
                role="listbox"
                aria-multiselectable
                aria-label={t('popup.files')}
                className={isList ? 'space-y-1 pb-1' : 'grid grid-cols-3 gap-1 pb-1'}
              >
                {files.map((file, index) => (
                  <div
                    key={file.id}
                    ref={(el) => {
                      fileRefs.current[file.id] = el;
                    }}
                    role="option"
                    aria-selected={selectedFiles.has(file.id.toString())}
                    tabIndex={index === activeIndex ? 0 : -1}
                    className={cn(
                      'p-1 rounded cursor-grab touch-pan-y active:cursor-grabbing outline-none focus-visible:ring-2 focus-visible:ring-ring',
                      isList ? 'flex items-center gap-2' : 'flex flex-col items-center',
                      selectedFiles.has(file.id.toString()) &&
                        'bg-primary/15 ring-1 ring-inset ring-primary/40',
                      !file.available && 'cursor-default text-muted-foreground opacity-60',
                    )}
                    title={file.available ? undefined : t('popup.offlineHint')}
                    onFocus={() => setActiveIndex(index)}
                    onClick={(e) => handleFileClick(file.id.toString(), e)}
                    onDoubleClick={() => openFile(file)}
                    onContextMenu={() => handleRowContextMenu(file.id.toString())}
                    onKeyDown={(e) => handleRowKeyDown(e, index)}
                    onPointerDown={(e) => handlePointerDown(e, file)}
                    // Holdem starts the drag itself (useNativeDrag); the
                    // page's own drag of a thumbnail would compete with it.
                    onDragStart={(e) => e.preventDefault()}
                  >
                    <div
                      data-drag-image
                      className={cn(
                        'relative flex items-center justify-center',
                        isList ? 'w-8 h-8 flex-shrink-0' : 'w-12 h-12 mb-1',
                      )}
                    >
                      <FileThumb file={file} />
                      {!file.available && (
                        <CloudOff
                          className="absolute -bottom-1 -end-1 h-3 w-3 text-muted-foreground"
                          aria-label={t('popup.offline')}
                        />
                      )}
                      {file.pinned && (
                        <Pin
                          className="absolute -end-1 -top-1 h-3 w-3 rotate-45 fill-primary text-primary"
                          aria-label={t('popup.pinned')}
                        />
                      )}
                    </div>
                    <div className={isList ? 'flex-grow min-w-0' : 'w-full text-center'}>
                      {renaming?.id === file.id ? (
                        <input
                          autoFocus
                          aria-label={t('popup.rename')}
                          className="h-6 w-full rounded border border-input bg-background px-1 text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring"
                          value={renaming.draft}
                          maxLength={80}
                          onFocus={(event) => event.target.select()}
                          onChange={(event) => setRenaming({ id: file.id, draft: event.target.value })}
                          onBlur={() => finishRename(true)}
                          onPointerDown={(event) => event.stopPropagation()}
                          onClick={(event) => event.stopPropagation()}
                          onDoubleClick={(event) => event.stopPropagation()}
                          onKeyDown={(event) => {
                            // Keys typed here are the name, not shortcuts.
                            event.stopPropagation();
                            if (event.key === 'Enter') finishRename(true);
                            if (event.key === 'Escape') finishRename(false);
                          }}
                        />
                      ) : (
                        <p
                          className={cn(
                            'text-xs font-medium truncate',
                            file.available ? 'text-foreground' : 'text-muted-foreground',
                          )}
                          title={file.available ? file.name : undefined}
                        >
                          {file.name}
                        </p>
                      )}
                      {!isList && (
                        <span className="text-[11px] text-muted-foreground">{sizeLabel(file)}</span>
                      )}
                    </div>
                    {isList && (
                      <span className="text-[11px] text-muted-foreground flex-shrink-0">
                        {sizeLabel(file)}
                      </span>
                    )}
                  </div>
                ))}
              </div>
              {files.length === 0 && (
                <p className="px-6 py-8 text-center text-xs text-muted-foreground">{t('popup.empty')}</p>
              )}
            </ContextMenu.Trigger>
            <ContextMenu.Portal>
              <ContextMenu.Content
                collisionPadding={4}
                // Closing gives focus back to the row; while a name is being
                // typed, that would end the rename at once.
                onCloseAutoFocus={(event) => {
                  if (renamingRef.current) event.preventDefault();
                }}
                className="max-h-[var(--radix-context-menu-content-available-height)] min-w-[200px] overflow-y-auto rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md"
              >
                {selectedFiles.size > 1 ? (
                  <ContextMenu.Label className="px-2 py-1 text-[11px] font-semibold text-muted-foreground">
                    {t('popup.itemsSelected', { count: selectedFiles.size })}
                  </ContextMenu.Label>
                ) : (
                  <>
                    <ContextMenu.Item
                      onClick={() => firstSelected && setPreviewId(firstSelected.id)}
                      disabled={!firstSelected?.available}
                      className={MENU_ITEM}
                    >
                      {t('preview.action')}
                      <Hint keys={t('keys.space')} />
                    </ContextMenu.Item>
                    <ContextMenu.Item
                      onClick={() => firstSelected && openFile(firstSelected)}
                      disabled={!firstSelected?.available}
                      className={MENU_ITEM}
                    >
                      {t('popup.open')}
                      <Hint keys={t('keys.enter')} />
                    </ContextMenu.Item>
                    <ContextMenu.Item
                      onClick={() => firstSelected && revealFile(firstSelected.id)}
                      disabled={!firstSelected?.available}
                      className={MENU_ITEM}
                    >
                      {t('popup.showInFolder')}
                    </ContextMenu.Item>
                    <ContextMenu.Item
                      onClick={() => startRename(firstSelected)}
                      disabled={!firstSelected?.available}
                      className={MENU_ITEM}
                    >
                      {t('popup.rename')}
                      <Hint keys="F2" />
                    </ContextMenu.Item>
                  </>
                )}
                <ContextMenu.Separator className="my-1 h-px bg-border" />
                <ContextMenu.Item
                  onClick={copySelected}
                  disabled={!anySelectedAvailable}
                  className={MENU_ITEM}
                >
                  {t('popup.copy')}
                  <Hint keys={t('keys.copy')} />
                </ContextMenu.Item>
                <ContextMenu.Item
                  onClick={copyPaths}
                  disabled={selectedFiles.size === 0}
                  className={MENU_ITEM}
                >
                  {t('popup.copyPath', { count: selectedFiles.size })}
                </ContextMenu.Item>
                <ContextMenu.Item
                  onClick={copySelectedToFolder}
                  disabled={!anySelectedAvailable}
                  className={MENU_ITEM}
                >
                  {t('actions.copyToFolder')}
                </ContextMenu.Item>
                <ContextMenu.Item
                  onClick={compressSelected}
                  disabled={!anySelectedAvailable}
                  className={MENU_ITEM}
                >
                  {t('actions.compress')}
                </ContextMenu.Item>
                {selectedImages.length > 0 && (
                  <ContextMenu.Sub>
                    <ContextMenu.SubTrigger className={MENU_ITEM}>
                      {t('image.menu')}
                      <ChevronRight className="h-3 w-3 rtl:rotate-180" />
                    </ContextMenu.SubTrigger>
                    <ContextMenu.Portal>
                      <ContextMenu.SubContent
                        collisionPadding={4}
                        className="min-w-[170px] rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md"
                      >
                        {!selectedImages.every((file) => file.file_type.toLowerCase() === 'png') && (
                          <ContextMenu.Item
                            className={MENU_ITEM}
                            onClick={() => transformImages('png', { kind: 'none' })}
                          >
                            {t('image.toPng')}
                          </ContextMenu.Item>
                        )}
                        {!selectedImages.every((file) =>
                          ['jpg', 'jpeg'].includes(file.file_type.toLowerCase()),
                        ) && (
                          <ContextMenu.Item
                            className={MENU_ITEM}
                            onClick={() => transformImages('jpeg', { kind: 'none' })}
                          >
                            {t('image.toJpeg')}
                          </ContextMenu.Item>
                        )}
                        <ContextMenu.Separator className="my-1 h-px bg-border" />
                        {RESIZES.map((resize) => (
                          <ContextMenu.Item
                            key={`${resize.kind}-${resize.value}`}
                            className={MENU_ITEM}
                            onClick={() => transformImages('same', resize)}
                          >
                            {resize.kind === 'percent'
                              ? t('image.percent', { value: resize.value })
                              : t('image.fit', { value: resize.value })}
                          </ContextMenu.Item>
                        ))}
                      </ContextMenu.SubContent>
                    </ContextMenu.Portal>
                  </ContextMenu.Sub>
                )}
                <ContextMenu.Separator className="my-1 h-px bg-border" />
                <ContextMenu.Item
                  onClick={() => setPinned(!allSelectedPinned)}
                  disabled={selectedFiles.size === 0}
                  className={MENU_ITEM}
                >
                  {t(allSelectedPinned ? 'popup.unpin' : 'popup.pin')}
                </ContextMenu.Item>
                <ContextMenu.Sub>
                  <ContextMenu.SubTrigger disabled={selectedFiles.size === 0} className={MENU_ITEM}>
                    {t('shelves.moveTo')}
                    <ChevronRight className="h-3 w-3 rtl:rotate-180" />
                  </ContextMenu.SubTrigger>
                  <ContextMenu.Portal>
                    <ContextMenu.SubContent
                      collisionPadding={4}
                      className="min-w-[150px] rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md"
                    >
                      {shelves
                        .filter((shelf) => shelf.id !== activeShelf)
                        .map((shelf) => (
                          <ContextMenu.Item
                            key={shelf.id}
                            className={MENU_ITEM}
                            onClick={() => moveSelected(shelf.id)}
                          >
                            <span className="truncate">{shelfName(shelf, t)}</span>
                          </ContextMenu.Item>
                        ))}
                      <ContextMenu.Item className={MENU_ITEM} onClick={moveSelectedToNewShelf}>
                        {t('shelves.newShelfWithItems')}
                      </ContextMenu.Item>
                    </ContextMenu.SubContent>
                  </ContextMenu.Portal>
                </ContextMenu.Sub>
                <ContextMenu.Separator className="my-1 h-px bg-border" />
                <ContextMenu.Item
                  onClick={handleRemoveSelectedFiles}
                  disabled={selectedFiles.size === 0}
                  className={MENU_ITEM}
                >
                  {t('popup.removeSelected')}
                  <Hint keys={t('keys.delete')} />
                </ContextMenu.Item>
              </ContextMenu.Content>
            </ContextMenu.Portal>
          </ContextMenu.Root>
        </div>
      </SimpleBar>
      {!notice && files.length > 0 && (
        <p className="mt-1 flex shrink-0 gap-2 px-1 text-[11px] text-muted-foreground" aria-live="polite">
          <span>
            {selectedFiles.size > 0
              ? t('popup.itemsSelected', { count: selectedFiles.size })
              : t('app.files', { count: files.length })}
          </span>
          {totalSize && <span>{totalSize}</span>}
          <span className="ms-auto">{t('preview.hint')}</span>
        </p>
      )}
      {/* Always on the page, so screen readers announce each new message. */}
      <div role="status" className="sr-only">
        {notice?.text ?? ''}
      </div>
      {notice && (
        <div
          className="mt-2 flex shrink-0 items-center justify-between gap-2 rounded-md bg-secondary px-3 py-1.5 text-xs text-secondary-foreground"
          onMouseEnter={() => window.clearTimeout(noticeTimer.current)}
          onMouseLeave={startNoticeTimer}
          onFocus={() => window.clearTimeout(noticeTimer.current)}
          onBlur={startNoticeTimer}
        >
          <span aria-hidden>{notice.text}</span>
          {notice.onUndo && (
            <Button
              size="sm"
              variant="ghost"
              className="h-6 px-2 text-xs font-semibold underline underline-offset-2"
              onClick={notice.onUndo}
            >
              {t('app.undo')}
            </Button>
          )}
        </div>
      )}
      {previewFile && (
        <PreviewPane
          file={previewFile}
          position={t('preview.position', { index: previewIndex + 1, count: files.length })}
          sizeLabel={sizeLabel(previewFile)}
          onClose={() => {
            setPreviewId(null);
            fileRefs.current[previewFile.id]?.focus();
          }}
        />
      )}
    </div>
  );
};

export default PopupWindow;
