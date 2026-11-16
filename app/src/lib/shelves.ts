import { invoke } from '@tauri-apps/api/core';
import type { TFunction } from 'i18next';
import { Shelf } from '@/types';

/** A shelf's display name: its own, or the translated default for the first one. */
export const shelfName = (shelf: Shelf, t: TFunction): string =>
  shelf.name || t('shelves.default');

/** Adds a shelf named "Shelf N" (the first free N) and makes it active. */
export const createShelf = (shelves: Shelf[], t: TFunction): Promise<number> => {
  const taken = new Set(shelves.map((shelf) => shelfName(shelf, t).toLowerCase()));
  let n = shelves.length + 1;
  while (taken.has(t('shelves.newName', { n }).toLowerCase())) n += 1;
  return invoke<number>('create_shelf', { name: t('shelves.newName', { n }) });
};

export const setActiveShelf = (id: number): Promise<void> =>
  invoke<void>('set_active_shelf', { id }).catch((error) =>
    console.error('Failed to switch shelf:', error),
  );

/** The shelf `step` places after the active one, wrapping around. */
export const cycleShelf = (shelves: Shelf[], active: number, step: 1 | -1): void => {
  if (shelves.length < 2) return;
  const index = shelves.findIndex((shelf) => shelf.id === active);
  const next = shelves[(index + step + shelves.length) % shelves.length];
  setActiveShelf(next.id);
};

/** Reopens a closed shelf (the most recent one when `id` is left out). */
export const reopenShelf = (id?: number): Promise<void> =>
  invoke<number>('reopen_shelf', { id: id ?? null })
    .then(() => undefined)
    .catch((error) => console.error('Failed to reopen the shelf:', error));
