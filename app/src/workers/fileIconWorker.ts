import { invoke } from '@tauri-apps/api/core';

self.onmessage = async (event) => {
  const { filePath } = event.data;
  try {
    console.log('filePath', filePath);
    const iconBase64: string = await invoke('get_file_icon_base64', { filePath });
    return iconBase64;
  } catch (error) {
    self.postMessage({ error: 'Error fetching file icon' });
  }
};