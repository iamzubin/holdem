export interface FilePreview {
  id: number;
  name: string;
  preview: string;
  type: 'file' | 'folder';
  size: number;
  path: string;
  icon: string;
}

export interface FileWithPath extends File {
  path: string;
}