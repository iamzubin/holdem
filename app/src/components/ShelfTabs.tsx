import { createShelf, reopenShelf, setActiveShelf, shelfName } from '@/lib/shelves';
import { MENU_ITEM } from '@/components/ui/menu';
import { cn } from '@/lib/utils';
import { ClosedShelf, Shelf } from '@/types';
import * as ContextMenu from '@radix-ui/react-context-menu';
import { invoke } from '@tauri-apps/api/core';
import { ChevronRight, Plus } from 'lucide-react';
import React, { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';


interface ShelfTabsProps {
  shelves: Shelf[];
  active: number;
  /** Item count per shelf id. */
  counts: Map<number, number>;
  /** Recently closed shelves, newest first. */
  closed: ClosedShelf[];
  /** Shelf `id` was closed with `count` items on it (it can be reopened). */
  onDeleted: (id: number, count: number) => void;
}

/**
 * The shelves as tabs: click to switch, double-click (or F2) to rename,
 * right-click to rename or delete, + to add one.
 */
export const ShelfTabs: React.FC<ShelfTabsProps> = ({
  shelves,
  active,
  counts,
  closed,
  onDeleted,
}) => {
  const { t } = useTranslation();
  const [renaming, setRenaming] = useState<number | null>(null);
  const [draft, setDraft] = useState('');
  // Lets the blur that follows Enter or Esc see that the edit is over.
  const renamingRef = useRef(renaming);
  renamingRef.current = renaming;
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (renaming !== null) inputRef.current?.select();
  }, [renaming]);

  const startRename = (shelf: Shelf) => {
    setDraft(shelfName(shelf, t));
    setRenaming(shelf.id);
  };

  const finishRename = (save: boolean) => {
    const id = renamingRef.current;
    if (id === null) return;
    renamingRef.current = null;
    setRenaming(null);
    // Focus goes back to the tab, not to the page.
    window.setTimeout(() => document.getElementById(`shelf-tab-${id}`)?.focus(), 0);
    const shelf = shelves.find((s) => s.id === id);
    // Unchanged: the first shelf keeps following the language's default name.
    if (!save || !shelf || draft.trim() === '' || draft.trim() === shelfName(shelf, t)) return;
    invoke('rename_shelf', { id, name: draft }).catch((error) =>
      console.error('Failed to rename shelf:', error),
    );
  };

  const deleteShelf = (id: number) => {
    invoke<number>('delete_shelf', { id })
      .then((count) => onDeleted(id, count))
      .catch((error) => console.error('Failed to delete shelf:', error));
  };

  const onTabKeyDown = (event: React.KeyboardEvent<HTMLButtonElement>, index: number) => {
    let next: number | null = null;
    const forward = document.documentElement.dir === 'rtl' ? 'ArrowLeft' : 'ArrowRight';
    if (event.key === 'ArrowRight' || event.key === 'ArrowLeft') {
      next = (index + (event.key === forward ? 1 : -1) + shelves.length) % shelves.length;
    } else if (event.key === 'F2') {
      // The tab's own keys: the item list's F2 and Delete must not act too.
      event.preventDefault();
      event.stopPropagation();
      startRename(shelves[index]);
      return;
    }
    if (next === null) return;
    event.preventDefault();
    event.stopPropagation();
    setActiveShelf(shelves[next].id);
    // Focus follows the selected tab.
    window.setTimeout(() => document.getElementById(`shelf-tab-${shelves[next].id}`)?.focus(), 0);
  };

  return (
    <div className="flex min-w-0 flex-1 items-center gap-0.5">
      <div
        role="tablist"
        aria-label={t('shelves.title')}
        className="flex min-w-0 items-center gap-0.5 overflow-x-auto [scrollbar-width:none]"
      >
        {shelves.map((shelf, index) =>
          renaming === shelf.id ? (
            <input
              key={shelf.id}
              ref={inputRef}
              aria-label={t('shelves.rename')}
              className="h-7 w-28 shrink-0 rounded-md border border-input bg-background px-2 text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring"
              value={draft}
              maxLength={40}
              onChange={(event) => setDraft(event.target.value)}
              onBlur={() => finishRename(true)}
              onKeyDown={(event) => {
                // Keys typed here are the name, not shortcuts.
                event.stopPropagation();
                if (event.key === 'Enter') finishRename(true);
                if (event.key === 'Escape') finishRename(false);
              }}
            />
          ) : (
            <ContextMenu.Root key={shelf.id}>
              <ContextMenu.Trigger asChild>
                <button
                  id={`shelf-tab-${shelf.id}`}
                  type="button"
                  role="tab"
                  aria-selected={shelf.id === active}
                  tabIndex={shelf.id === active ? 0 : -1}
                  title={t('shelves.tabHint')}
                  className={cn(
                    'flex h-7 max-w-[140px] shrink-0 items-center gap-1 rounded-md px-2 text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring',
                    shelf.id === active
                      ? 'bg-secondary font-semibold text-foreground'
                      : 'text-muted-foreground hover:bg-secondary/60 hover:text-foreground',
                  )}
                  onClick={() => setActiveShelf(shelf.id)}
                  onDoubleClick={() => startRename(shelf)}
                  onKeyDown={(event) => onTabKeyDown(event, index)}
                >
                  <span className="truncate">{shelfName(shelf, t)}</span>
                  <span className="text-[11px] font-normal text-muted-foreground" aria-hidden>
                    {counts.get(shelf.id) ?? 0}
                  </span>
                  <span className="sr-only">{t('app.files', { count: counts.get(shelf.id) ?? 0 })}</span>
                </button>
              </ContextMenu.Trigger>
              <ContextMenu.Portal>
                <ContextMenu.Content className="min-w-[160px] rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md">
                  <ContextMenu.Item className={MENU_ITEM} onSelect={() => startRename(shelf)}>
                    {t('shelves.rename')}
                  </ContextMenu.Item>
                  <ContextMenu.Item
                    className={MENU_ITEM}
                    disabled={shelves.length < 2}
                    onSelect={() => deleteShelf(shelf.id)}
                  >
                    {t('shelves.delete')}
                  </ContextMenu.Item>
                  {closed.length > 0 && (
                    <>
                      <ContextMenu.Separator className="my-1 h-px bg-border" />
                      <ContextMenu.Sub>
                        <ContextMenu.SubTrigger className={MENU_ITEM}>
                          {t('shelves.reopen')}
                          <ChevronRight className="h-3 w-3 rtl:rotate-180" />
                        </ContextMenu.SubTrigger>
                        <ContextMenu.Portal>
                          <ContextMenu.SubContent
                            collisionPadding={4}
                            className="min-w-[160px] rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md"
                          >
                            {closed.map((entry, index) => (
                              <ContextMenu.Item
                                key={entry.shelf.id}
                                className={MENU_ITEM}
                                title={index === 0 ? t('keys.reopenShelf') : undefined}
                                onSelect={() => reopenShelf(entry.shelf.id)}
                              >
                                <span className="truncate">{shelfName(entry.shelf, t)}</span>
                                <span className="text-[11px] text-muted-foreground">
                                  {t('app.files', { count: entry.items.length })}
                                </span>
                              </ContextMenu.Item>
                            ))}
                          </ContextMenu.SubContent>
                        </ContextMenu.Portal>
                      </ContextMenu.Sub>
                    </>
                  )}
                </ContextMenu.Content>
              </ContextMenu.Portal>
            </ContextMenu.Root>
          ),
        )}
      </div>
      <button
        type="button"
        className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-muted-foreground outline-none hover:bg-secondary hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
        aria-label={t('shelves.new')}
        title={t('shelves.new')}
        onClick={() =>
          createShelf(shelves, t).catch((error) => console.error('Failed to add a shelf:', error))
        }
      >
        <Plus className="h-4 w-4" />
      </button>
    </div>
  );
};
