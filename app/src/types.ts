/** A shelf entry. Mirrors `FileMetadata` in src-tauri/src/file.rs. */
export interface FilePreview {
  id: number;
  name: string;
  path: string;
  /** Bytes; always 0 for folders, whose size is not computed. */
  size: number;
  /** "folder", or the file's extension ("unknown" when it has none). */
  file_type: string;
  /** "note" for a saved text snippet (dragged out as text), else "file". */
  kind: 'file' | 'note';
  /** Pinned items stay when the shelf is cleared. */
  pinned: boolean;
  /** False while the item's drive or network location is offline. */
  available: boolean;
  /** Id of the shelf the item is on. */
  shelf: number;
  /** Holdem saved this file itself (attachment, image, text, link): it can be renamed. */
  owned: boolean;
}

/** A named shelf. Mirrors `Shelf` in src-tauri/src/shelves.rs. */
export interface Shelf {
  id: number;
  /** Empty for the first shelf, shown under its translated default name. */
  name: string;
}

/** A closed shelf that can be reopened. Mirrors `ClosedShelf` in shelves.rs. */
export interface ClosedShelf {
  shelf: Shelf;
  items: { path: string; pinned: boolean }[];
  /** Seconds since 1970. */
  closedAt: number;
}

export interface ShelfSet {
  shelves: Shelf[];
  active: number;
  /** Newest first. */
  closed: ClosedShelf[];
}

/** Images the backend can convert and resize (image_tools.rs). */
const IMAGE_TYPES = ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp', 'tif', 'tiff', 'ico'];
export const isConvertibleImage = (file: FilePreview): boolean =>
  file.available && IMAGE_TYPES.includes(file.file_type.toLowerCase());

export const isFolder = (file: FilePreview): boolean => file.file_type === 'folder';
