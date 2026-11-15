/** A shelf entry. Mirrors `FileMetadata` in src-tauri/src/file.rs. */
export interface FilePreview {
  id: number;
  name: string;
  path: string;
  /** Bytes; always 0 for folders, whose size is not computed. */
  size: number;
  /** "folder", or the file's extension ("unknown" when it has none). */
  file_type: string;
}

export const isFolder = (file: FilePreview): boolean => file.file_type === 'folder';
