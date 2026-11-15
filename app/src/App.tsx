import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog';
import { useFileManagement } from '@/hooks/useFileManagement';
import { hideMainWindow } from '@/lib/windowUtils';
import { invoke } from '@tauri-apps/api/core';
import { ChevronDown, Download, Settings, X } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { StackedIcons } from './components/StackedIcons';
import { useTheme } from '@/components/theme-provider';

const MAX_FALLBACK_DROP_BYTES = 64 * 1024 * 1024;

function App() {
  const { t } = useTranslation();
  const { theme } = useTheme();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const { files, clearFiles, refreshFiles } = useFileManagement();

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
        console.warn(`Skipped ${dropped.length - copyable.length} dropped file(s) over 64 MB`);
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
    [refreshFiles],
  );

  const openPopup = () => {
    invoke('open_popup_window', { theme }).catch((err) => console.error(err));
  };

  const handleContextMenu = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      // The menu's only action is Clear, so there is nothing to show when empty.
      if (files.length > 0) setIsModalOpen(true);
    },
    [files.length],
  );

  const openSettings = () => {
    invoke('open_settings_window', { theme }).catch((err) => console.error(err));
  };

  const handleClear = () => {
    clearFiles();
    setIsModalOpen(false);
  };

  return (
    <div
      className="fixed inset-0 text-foreground flex flex-col bg-background p-2 focus:outline-none"
      onContextMenu={handleContextMenu}
      tabIndex={0}
      onDragEnter={preventDefault}
      onDragLeave={preventDefault}
      onDragOver={handleDragOver}
      onDrop={handleDrop}
    >
      {/* Handle and Title Bar */}
      <div
        className="relative flex justify-end items-center h-5"
        data-tauri-drag-region
        onDragStart={(e) => {
          e.preventDefault();
        }}
      >
        <div className="absolute left-1/2 transform -translate-x-1/2 top-1/2 -translate-y-1/2">
          <div className="w-10 h-0.5 bg-foreground rounded-full" data-tauri-drag-region></div>
        </div>
        <Button
          variant="ghost"
          size="icon"
          className="text-foreground hover:bg-gray-500 hover:text-background rounded h-5 w-5"
          onClick={openSettings}
          aria-label={t('settings.title')}
        >
          <Settings className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="text-foreground hover:bg-red-500 hover:text-background rounded h-5 w-5"
          onClick={hideMainWindow}
          aria-label={t('common.close')}
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
              <span className="text-[8px]">{t('app.dropHere')}</span>
            </div>
          )}
        </div>

        {/* Dropdown Button at the Bottom */}
        <div className="flex justify-center items-center mt-1">
          <Button onClick={openPopup} variant="secondary">
            <span>{t('app.files', { count: files.length })}</span>
            <ChevronDown className="h-2 w-2 ms-1" />
          </Button>
        </div>
      </>

      <Dialog open={isModalOpen} onOpenChange={setIsModalOpen}>
        <DialogContent className="rounded-md p-0 mt-2 w-[90vw]" aria-describedby={undefined}>
          <DialogTitle className="sr-only">{t('app.contextMenuTitle')}</DialogTitle>
          <div className="flex flex-col items-start text-foreground">
            {files.length > 0 && (
              <Button
                className="w-full text-left justify-start hover:bg-secondary transition-colors"
                variant="ghost"
                onClick={handleClear}
              >
                <X className="h-4 w-4 me-2" />
                {t('app.clear')}
              </Button>
            )}
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}

export default App;
