import { useState, useCallback, useEffect } from 'react';
import { FilePreview } from '../types.ts';
import { emit, listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export const useFileManagement = () => {
  const [files, setFiles] = useState<FilePreview[]>([]);
  const [_files, set_files] = useState<FilePreview[]>([]);

  const fetchFiles = useCallback(async () => {
    try {
      const fetchedFiles: FilePreview[] = await invoke('get_files');
      setFiles(fetchedFiles);
    } catch (error) {
      console.error('Error fetching files:', error);
    }
  }, []);

  useEffect(() => {

    fetchFiles();

    const unlistenFileAdded = listen('file_added', (event: any) => {
      console.log('file_added', event);
      setFiles(prevFiles => [...prevFiles, event.payload]);
    });

    const unlistenFileRemoved = listen('file_removed', (event: any) => {
      setFiles(prevFiles => prevFiles.filter(file => file.id !== event.payload));
    });

    const unlistenFileRenamed = listen('file_renamed', (event: any) => {
      setFiles(prevFiles => prevFiles.map(file => 
        file.id === event.payload.id ? { ...file, name: event.payload.newName } : file
      ));
    });

    const unlistenFilesUpdated = listen('files_updated', () => {
      fetchFiles();
    });

    return () => {
      unlistenFileAdded.then(f => f());
      unlistenFileRemoved.then(f => f());
      unlistenFileRenamed.then(f => f());
      unlistenFilesUpdated.then(f => f());
    };
  }, [fetchFiles, files.length]);

  const addFiles = useCallback(async (newFiles: FilePreview[]) => {
    const paths = newFiles.map(file => file.path);
    try {
      await invoke('add_files', { files: paths });
      // The backend will emit a 'file_added' event, so we don't need to update the state here
    } catch (error) {
      console.error('Error adding files:', error);
    }
  }, []);

  const removeFile = useCallback(async (id: number) => {
    try {
      await invoke('remove_file', { fileId: id });
      // The backend will emit a 'file_removed' event, so we don't need to update the state here
    } catch (error) {
      console.error('Error removing file:', error);
    }
  }, []);

  const renameFile = useCallback(async (id: number, newName: string) => {
    try {
      await invoke('rename_file', { fileId: id, newName });
      // The backend will emit a 'file_renamed' event, so we don't need to update the state here
    } catch (error) {
      console.error('Error renaming file:', error);
    }
  }, []);

  const getFileIcon = useCallback(async (filePath: string): Promise<string> => {
    try {
      const iconBase64: string = await invoke('get_file_icon_base64', { filePath });
      return iconBase64;
    } catch (error) {
      console.error('Error fetching file icon:', error);
      throw error;
    }
  }, []);

  return { files, addFiles, removeFile, renameFile, getFileIcon };
};