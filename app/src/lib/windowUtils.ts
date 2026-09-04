import { getCurrentWindow } from '@tauri-apps/api/window';

export const closeWindow = () => getCurrentWindow().hide();