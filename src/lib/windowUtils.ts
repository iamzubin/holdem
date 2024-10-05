import { getCurrentWindow } from '@tauri-apps/api/window';

export const minimizeWindow = () => getCurrentWindow().minimize();
export const maximizeWindow = () => getCurrentWindow().maximize();
export const closeWindow = () => getCurrentWindow().hide();