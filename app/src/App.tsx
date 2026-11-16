import { Button } from '@/components/ui/button';
import { useFileManagement } from '@/hooks/useFileManagement';
import { hideMainWindow } from '@/lib/windowUtils';
import { invoke } from '@tauri-apps/api/core';
import { cn } from '@/lib/utils';
import { Check, ChevronDown, Download, Plus, Settings, X } from 'lucide-react';
import { createShelf, cycleShelf, reopenShelf, setActiveShelf, shelfName } from '@/lib/shelves';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { StackedIcons } from './components/StackedIcons';
import { useTheme } from '@/components/theme-provider';
import { useTauriEvent } from '@/hooks/useTauriEvent';
import { ShelfNotice, shelfNoticeText } from '@/lib/notices';

const MAX_FALLBACK_DROP_BYTES = 64 * 1024 * 1024;
const NOTICE_MS = 6000;
/** Failures stay longer: they explain what to do next. */
const ERROR_NOTICE_MS = 12000;

/** A short message in place of the files button, optionally with Undo. */
interface Notice {
  text: string;
  /** The whole explanation, when `text` is a shortened one. */
  detail?: string;
  error?: boolean;
  undo?: boolean;
}

function App() {
  const { t, i18n } = useTranslation();
  const { theme } = useTheme();
  const [isDropHover, setIsDropHover] = useState(false);
  const { files, allFiles, shelves, activeShelf, closedShelves, clearFiles, refreshFiles } =
    useFileManagement();
  const [notice, setNotice] = useState<Notice | null>(null);
  // The right-click menu, or the shelf switcher's list of shelves.
  const [menu, setMenu] = useState<'actions' | 'shelves' | null>(null);
  const menuOpen = menu !== null;
  const setMenuOpen = (open: boolean) => setMenu(open ? 'actions' : null);
  const current = shelves.find((shelf) => shelf.id === activeShelf);
  const multipleShelves = shelves.length > 1;
  const newShelf = () => {
    createShelf(shelves, t).catch((error) => console.error('Failed to add a shelf:', error));
  };
  // Clear leaves pinned items on the shelf.
  const unpinnedCount = files.filter((file) => !file.pinned).length;
  const menuRef = useRef<HTMLDivElement>(null);

  // Keyboard users land on the first item; arrows move, Esc/Tab close.
  useEffect(() => {
    if (menuOpen) {
      menuRef.current?.querySelector<HTMLButtonElement>('[role^=menuitem]:not(:disabled)')?.focus();
    }
  }, [menuOpen]);

  const runMenu = (action: () => void) => {
    setMenuOpen(false);
    action();
  };

  const handleMenuKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    const items = Array.from(
      menuRef.current?.querySelectorAll<HTMLButtonElement>('[role^=menuitem]:not(:disabled)') ?? [],
    );
    const current = items.indexOf(document.activeElement as HTMLButtonElement);
    if (event.key === 'Escape' || event.key === 'Tab') {
      event.preventDefault();
      setMenuOpen(false);
    } else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      const step = event.key === 'ArrowDown' ? 1 : -1;
      items[(current + step + items.length) % items.length]?.focus();
    }
  };
  const noticeTimer = useRef<number | undefined>(undefined);

  // Pointing at or focusing the message pauses it; an opened explanation
  // stays until closed.
  const [noticeOpen, setNoticeOpen] = useState(false);
  const noticeMs = useRef(NOTICE_MS);
  const startNoticeTimer = useCallback(() => {
    window.clearTimeout(noticeTimer.current);
    noticeTimer.current = window.setTimeout(() => setNotice(null), noticeMs.current);
  }, []);
  const pauseNotice = () => window.clearTimeout(noticeTimer.current);
  const resumeNotice = () => {
    if (!noticeOpen) startNoticeTimer();
  };
  const showNotice = useCallback(
    (next: Notice) => {
      setNotice(next);
      setNoticeOpen(false);
      noticeMs.current = next.error ? ERROR_NOTICE_MS : NOTICE_MS;
      startNoticeTimer();
    },
    [startNoticeTimer],
  );
  const closeNotice = () => {
    window.clearTimeout(noticeTimer.current);
    setNotice(null);
    setNoticeOpen(false);
  };

  useEffect(() => () => window.clearTimeout(noticeTimer.current), []);

  const paste = () => {
    invoke('paste_clipboard').catch((error) => console.error('Paste failed:', error));
  };

  /** Puts every shelf item on the clipboard as files. */
  const copyAll = useCallback(() => {
    invoke<number>('copy_items_to_clipboard', { ids: files.map((f) => f.id) })
      .then((count) =>
        showNotice({ text: count > 0 ? t('popup.copiedFiles', { count }) : t('popup.offlineHint') }),
      )
      .catch((error) => console.error('Copy failed:', error));
  }, [files, showNotice, t]);

  // Ctrl+V adds the clipboard, exactly like dropping it; Ctrl+C copies the
  // whole shelf as files; Ctrl+Tab switches shelves; Ctrl+Shift+T reopens
  // the last closed shelf.
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey)) return;
      const key = event.key.toLowerCase();
      if (event.key === 'Tab') {
        event.preventDefault();
        cycleShelf(shelves, activeShelf, event.shiftKey ? -1 : 1);
      } else if (event.shiftKey && key === 't') {
        event.preventDefault();
        if (closedShelves.length > 0) reopenShelf();
      } else if (key === 'v') {
        event.preventDefault();
        paste();
      } else if (key === 'c' && files.length > 0) {
        event.preventDefault();
        copyAll();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [files, copyAll, shelves, activeShelf, closedShelves]);

  // The tray menu is built natively before any page loads; give it the
  // user's language once translations are ready, and on every change.
  useEffect(() => {
    const labels = {
      show: t('tray.show'),
      paste: t('tray.paste'),
      settings: t('tray.settings'),
      checkUpdates: t('tray.checkUpdates'),
      quit: t('tray.quit'),
    };
    invoke('set_tray_labels', { labels }).catch((error) =>
      console.error('Failed to translate the tray menu:', error),
    );
  }, [i18n.resolvedLanguage, t]);

  // What happened to a drop (why items were not added), and items that
  // went missing.
  useTauriEvent<ShelfNotice>('shelf-notice', (event) => {
    const { short, full, error } = shelfNoticeText(event, t);
    showNotice({ text: short, detail: full === short ? undefined : full, error });
  });

  // The native drop target reports drags over the shelf (see drop_target.rs).
  useTauriEvent<boolean>('drop-hover', setIsDropHover);

  // Check analytics consent on mount
  useEffect(() => {
    const checkConsent = async () => {
      try {
        const configExists = await invoke<boolean>('check_config_exists');

        if (!configExists) {
          // Add a small delay to ensure main window is fully loaded
          setTimeout(() => {
            invoke('open_consent_window').catch((error) =>
              console.error('Failed to open consent window:', error),
            );
          }, 500);
        }
      } catch (error) {
        console.error('Failed to check config existence:', error);
        // If we can't check, don't show consent window to be safe
      }
    };

    checkConsent();
  }, []);

  const preventDefault = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDragOver = useCallback(
    (e: React.DragEvent<HTMLDivElement>) => {
      preventDefault(e);
      e.dataTransfer.dropEffect = 'copy';
    },
    [preventDefault],
  );

  // Fallback only. Drops are owned by the native HoldemDropTarget
  // (src-tauri/src/drop_target.rs), which replaces WebView2's drop target, so
  // these HTML drop events normally never fire. They do if that registration
  // failed or WebView2 recreated its window; then every file arrives here as
  // a plain File (no filesystem path), so copy its bytes into the shelf.
  const handleDrop = useCallback(
    async (e: React.DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      e.stopPropagation();

      const dropped = Array.from(e.dataTransfer.files);
      // Same per-file cap as native virtual files; reading a huge file into
      // a data URL would exhaust the webview's memory.
      const copyable = dropped.filter((file) => file.size <= MAX_FALLBACK_DROP_BYTES);
      if (copyable.length < dropped.length) {
        showNotice({ text: t('app.tooLarge', { count: dropped.length - copyable.length }) });
      }
      if (copyable.length === 0) return;

      invoke('mark_drop_received').catch(() => {});
      for (const file of copyable) {
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
              refreshFiles();
            } catch (err) {
              console.error('Failed to save dropped file', err);
              showNotice({ text: t('app.dropFailed', { count: 1 }) });
            }
            resolve();
          };
          reader.onerror = () => {
            // Folders cannot be read as files and end up here.
            console.error('Failed to read dropped file', reader.error);
            resolve();
          };
        });
        reader.readAsDataURL(file);
        await done;
      }
    },
    [refreshFiles, showNotice, t],
  );

  // Opened with the keyboard (detail 0 for Enter/Space, or from the menu),
  // the popup takes focus.
  const openPopup = (focus: boolean) => {
    invoke('open_popup_window', { theme, focus }).catch((err) => console.error(err));
  };

  const openSettings = () => {
    invoke('open_settings_window', { theme }).catch((err) => console.error(err));
  };

  const handleClear = async () => {
    const count = await clearFiles();
    if (count > 0) showNotice({ text: t('app.removed', { count }), undo: true });
  };

  const undoRemove = () => {
    setNotice(null);
    invoke('restore_removed').catch((error) => console.error('Failed to undo:', error));
  };

  return (
    <div
      className={cn(
        'fixed inset-0 text-foreground flex flex-col bg-background p-2 transition-colors',
        isDropHover && 'ring-2 ring-inset ring-primary',
      )}
      onDragEnter={preventDefault}
      onDragLeave={preventDefault}
      onDragOver={handleDragOver}
      onDrop={handleDrop}
      onContextMenu={(event) => {
        event.preventDefault();
        setMenuOpen(true);
      }}
    >
      {/* Tint while files are dragged over. An overlay, not a background
          swap: the window is transparent (Mica), so the page background
          must stay solid. */}
      {isDropHover && (
        <div aria-hidden className="pointer-events-none absolute inset-0 z-10 bg-primary/10" />
      )}
      {/* Handle and Title Bar */}
      <div
        className="relative flex justify-end items-center h-5"
        data-tauri-drag-region
        onDragStart={(e) => {
          e.preventDefault();
        }}
      >
        {multipleShelves && current ? (
          // Which shelf this is, and the way to switch.
          <button
            type="button"
            className="me-auto flex h-6 max-w-[96px] items-center gap-0.5 rounded px-1.5 text-xs font-medium hover:bg-accent"
            aria-haspopup="menu"
            aria-expanded={menu === 'shelves'}
            aria-label={t('shelves.switch', { name: shelfName(current, t) })}
            onClick={() => setMenu(menu === 'shelves' ? null : 'shelves')}
          >
            <span className="truncate">{shelfName(current, t)}</span>
            <ChevronDown className="h-3 w-3 shrink-0" />
          </button>
        ) : (
          <div className="absolute left-1/2 transform -translate-x-1/2 top-1/2 -translate-y-1/2">
            <div className="w-10 h-0.5 bg-foreground rounded-full" data-tauri-drag-region></div>
          </div>
        )}
        <Button
          variant="ghost"
          size="icon"
          className="text-foreground hover:bg-accent hover:text-accent-foreground rounded h-6 w-6"
          onClick={openSettings}
          aria-label={t('settings.title')}
          title={t('settings.title')}
        >
          <Settings className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="text-foreground hover:bg-accent hover:text-accent-foreground rounded h-6 w-6"
          onClick={hideMainWindow}
          aria-label={t('app.hide')}
          title={t('app.hide')}
        >
          <X className="h-4 w-4" />
        </Button>
      </div>

      <>
        {/* Main Content */}
        <div
          className="grow flex flex-col items-center justify-center space-y-1"
          onDragEnter={preventDefault}
          onDragOver={handleDragOver}
          onDragLeave={preventDefault}
          onDrop={handleDrop}
        >
          {files.length > 0 ? (
            <div className="relative w-10 h-10 flex items-center justify-center">
              <StackedIcons files={files} />
            </div>
          ) : (
            <div className="flex flex-col items-center">
              <Download className="h-5 w-5" />
              <span className="text-xs">{t('app.dropHere')}</span>
            </div>
          )}
        </div>

        {/* Dropdown Button at the Bottom */}
        {/* Messages sit above the button so the item count stays visible,
            and wrap instead of being cut off. */}
        {/* Always on the page, so screen readers announce each new message,
            in full. */}
        <div role="status" className="sr-only">
          {notice ? (notice.detail ?? notice.text) : ''}
        </div>
        {notice && (
          <div
            title={noticeOpen ? undefined : notice.detail}
            className="absolute inset-x-2 bottom-12 z-30 flex items-start gap-1 rounded-md border border-border bg-secondary px-2 py-1 text-[11px] leading-tight text-secondary-foreground shadow-sm"
            onMouseEnter={pauseNotice}
            onMouseLeave={resumeNotice}
            onFocus={pauseNotice}
            onBlur={resumeNotice}
          >
            {/* The shelf is small: a short message, and the whole
                explanation (what to do next) on request. Screen readers
                get the whole explanation at once. */}
            {/* An opened explanation can be longer than the shelf is tall. */}
            <span
              className="max-h-[calc(100vh-4rem)] min-w-0 flex-1 self-center overflow-y-auto"
              aria-hidden
            >
              {noticeOpen ? notice.detail : notice.text}
            </span>
            {notice.detail && !noticeOpen && (
              <Button
                size="sm"
                variant="ghost"
                className="h-6 shrink-0 px-1.5 text-[11px] font-semibold underline underline-offset-2"
                onClick={() => {
                  pauseNotice();
                  setNoticeOpen(true);
                }}
              >
                {t('common.details')}
              </Button>
            )}
            {noticeOpen && (
              <Button
                size="icon"
                variant="ghost"
                className="h-5 w-5 shrink-0"
                aria-label={t('common.close')}
                onClick={closeNotice}
              >
                <X className="h-3 w-3" />
              </Button>
            )}
            {notice.undo && (
              <Button
                size="sm"
                variant="ghost"
                className="h-6 shrink-0 px-1.5 text-[11px] font-semibold underline underline-offset-2"
                onClick={undoRemove}
              >
                {t('app.undo')}
              </Button>
            )}
          </div>
        )}
        <div className="flex min-h-9 justify-center items-center mt-1">
          {files.length > 0 && (
            <Button onClick={(event) => openPopup(event.detail === 0)} variant="secondary">
              <span>{t('app.files', { count: files.length })}</span>
              <ChevronDown className="h-2 w-2 ms-1" />
            </Button>
          )}
        </div>
      </>

      {menuOpen && (
        <>
          {/* Click-away layer; a right-click on it closes the menu too. */}
          <div
            className="fixed inset-0 z-40"
            onMouseDown={() => setMenuOpen(false)}
            onContextMenu={(event) => {
              event.preventDefault();
              setMenuOpen(false);
            }}
          />
          {/* Pinned inside the shelf: the window is too small for a menu at the cursor.
              Settings and Hide have their own buttons in the title bar. */}
          <div
            ref={menuRef}
            role="menu"
            aria-label={menu === 'shelves' ? t('shelves.title') : t('app.contextMenuTitle')}
            className="fixed inset-x-2 top-1 z-50 max-h-[calc(100vh-0.5rem)] overflow-y-auto rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md"
            onKeyDown={handleMenuKeyDown}
          >
            {menu === 'shelves' ? (
              <>
                {shelves.map((shelf) => (
                  <button
                    key={shelf.id}
                    role="menuitemradio"
                    aria-checked={shelf.id === activeShelf}
                    className="flex w-full items-center justify-between gap-3 rounded px-2 py-1 text-start text-xs outline-none hover:bg-secondary focus-visible:bg-secondary disabled:pointer-events-none disabled:opacity-40"
                    onClick={() => runMenu(() => setActiveShelf(shelf.id))}
                  >
                    <span className="flex min-w-0 items-center gap-1">
                      <Check className={cn('h-3 w-3 shrink-0', shelf.id !== activeShelf && 'invisible')} />
                      <span className="truncate">{shelfName(shelf, t)}</span>
                    </span>
                    <span className="text-[11px] text-muted-foreground">
                      {allFiles.filter((file) => file.shelf === shelf.id).length}
                    </span>
                  </button>
                ))}
                <div role="separator" className="my-1 h-px bg-border" />
                <button role="menuitem" className="flex w-full items-center justify-between gap-3 rounded px-2 py-1 text-start text-xs outline-none hover:bg-secondary focus-visible:bg-secondary disabled:pointer-events-none disabled:opacity-40" onClick={() => runMenu(newShelf)}>
                  <span className="flex items-center gap-1">
                    <Plus className="h-3 w-3" />
                    {t('shelves.new')}
                  </span>
                </button>
              </>
            ) : (
            <>
            <button
              role="menuitem"
              className="flex w-full items-center justify-between gap-3 rounded px-2 py-1 text-start text-xs outline-none hover:bg-secondary focus-visible:bg-secondary disabled:pointer-events-none disabled:opacity-40"
              disabled={files.length === 0}
              onClick={() => runMenu(() => openPopup(true))}
            >
              {t('app.showFiles')}
            </button>
            <button role="menuitem" className="flex w-full items-center justify-between gap-3 rounded px-2 py-1 text-start text-xs outline-none hover:bg-secondary focus-visible:bg-secondary disabled:pointer-events-none disabled:opacity-40" onClick={() => runMenu(paste)}>
              {t('app.paste')}
              <span className="text-[11px] text-muted-foreground">{t('keys.paste')}</span>
            </button>
            <button
              role="menuitem"
              className="flex w-full items-center justify-between gap-3 rounded px-2 py-1 text-start text-xs outline-none hover:bg-secondary focus-visible:bg-secondary disabled:pointer-events-none disabled:opacity-40"
              disabled={files.length === 0}
              onClick={() => runMenu(copyAll)}
            >
              {t('app.copyAll')}
              <span className="text-[11px] text-muted-foreground">{t('keys.copy')}</span>
            </button>
            <button
              role="menuitem"
              className="flex w-full items-center justify-between gap-3 rounded px-2 py-1 text-start text-xs outline-none hover:bg-secondary focus-visible:bg-secondary disabled:pointer-events-none disabled:opacity-40"
              disabled={unpinnedCount === 0}
              onClick={() => runMenu(handleClear)}
            >
              {t('app.clearShelf', { count: unpinnedCount })}
            </button>
            <div role="separator" className="my-1 h-px bg-border" />
            <button role="menuitem" className="flex w-full items-center justify-between gap-3 rounded px-2 py-1 text-start text-xs outline-none hover:bg-secondary focus-visible:bg-secondary disabled:pointer-events-none disabled:opacity-40" onClick={() => runMenu(newShelf)}>
              {t('shelves.new')}
            </button>
            </>
            )}
          </div>
        </>
      )}
    </div>
  );
}

export default App;
