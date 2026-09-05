import { invoke } from '@tauri-apps/api/core';

/** Stateless file-icon fetch — no React hook, safe to call per-icon. */
export const getFileIconBase64 = async (filePath: string): Promise<string> => {
  const iconBase64: string = await invoke('get_file_icon_base64', { filePath });
  return iconBase64;
};
