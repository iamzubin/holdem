import { useState, useCallback, useEffect } from 'react';
import { FilePreview } from '../types.ts';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export const useFileManagement = () => {
  const [files, setFiles] = useState<FilePreview[]>([]);

  const refreshFiles = useCallback(async () => {
    try {
      // Prune externally-deleted paths first (`get_files` is a pure clone
      // and would otherwise leave ghosts). No polling — this runs on mount,
      // on `files_updated`, and on explicit refreshes only.
      await invoke('refresh_file_list').catch(() => {});
      const fetchedFiles: FilePreview[] = await invoke('get_files');
      setFiles(fetchedFiles);
    } catch (error) {
      console.error('Error fetching files:', error);
    }
  }, []);

  useEffect(() => {
    refreshFiles();

    let unlisten: (() => void) | undefined;
    let cancelled = false;
    listen('files_updated', () => {
      refreshFiles();
    }).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [refreshFiles]);

  const clearFiles = useCallback(async () => {
    try {
      await invoke('clear_files');
    } catch (error) {
      console.error('Error clearing files:', error);
    }
  }, []);

  return { files, refreshFiles, clearFiles };
};