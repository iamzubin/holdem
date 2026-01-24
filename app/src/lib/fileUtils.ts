import { invoke } from "@tauri-apps/api/core";
import { FilePreview } from "../types.ts";

export const handleDragStart = async (e: React.DragEvent<HTMLDivElement>, file: FilePreview) => {
  e.preventDefault();
  e.stopPropagation();

  e.dataTransfer.setData('text/plain', file.path);
  e.dataTransfer.effectAllowed = 'copyMove';

  try {
    await invoke('start_drag', { filePath: file.path });
  } catch (error) {
    console.error('Error starting drag:', error);
  }
};

export const handleMultiFileDragStart = async (e: React.DragEvent<HTMLDivElement>, files: FilePreview[]) => {
  e.preventDefault();
  e.stopPropagation();

  const fileNames = files.map(file => file.path).join('\n');
  e.dataTransfer.setData('text/plain', fileNames);
  e.dataTransfer.effectAllowed = 'copyMove';

  try {
    await invoke('start_multi_drag', { filePaths: files.map(file => file.path) });
  } catch (error) {
    console.error('Error starting multi-file drag:', error);
  }
};