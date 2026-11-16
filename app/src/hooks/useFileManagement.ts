import { useState, useCallback, useEffect, useMemo } from 'react';
import { FilePreview, ShelfSet } from '../types.ts';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

const NO_SHELVES: ShelfSet = { shelves: [], active: 0, closed: [] };

/**
 * The shelf's items and the list of shelves. `files` holds the active
 * shelf's items only; `allFiles` holds every shelf's.
 */
export const useFileManagement = () => {
  const [allFiles, setAllFiles] = useState<FilePreview[]>([]);
  const [shelfSet, setShelfSet] = useState<ShelfSet>(NO_SHELVES);

  const refreshFiles = useCallback(async () => {
    try {
      const [fetchedFiles, fetchedShelves] = await Promise.all([
        invoke<FilePreview[]>('get_files'),
        invoke<ShelfSet>('get_shelves'),
      ]);
      setAllFiles(fetchedFiles);
      setShelfSet(fetchedShelves);
      // Then re-check the items in the background (deleted ones leave,
      // offline ones grey out). It can wait seconds on a dead network share,
      // so it is not awaited; any change arrives as `files_updated`. No
      // polling: this runs on mount, on `files_updated` and on refreshes.
      invoke('refresh_file_list').catch(() => {});
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

  /** Empties the active shelf (pinned items stay); resolves to how many left. */
  const clearFiles = useCallback(async (): Promise<number> => {
    try {
      return await invoke<number>('clear_files');
    } catch (error) {
      console.error('Error clearing files:', error);
      return 0;
    }
  }, []);

  // The same array until the items or the shelf change, so effects that
  // depend on it don't run on every render.
  const files = useMemo(
    () => allFiles.filter((file) => file.shelf === shelfSet.active),
    [allFiles, shelfSet.active],
  );

  return {
    files,
    allFiles,
    shelves: shelfSet.shelves,
    activeShelf: shelfSet.active,
    closedShelves: shelfSet.closed ?? [],
    refreshFiles,
    clearFiles,
  };
};
