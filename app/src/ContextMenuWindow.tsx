import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { Check, Trash2, X } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";

/** Emitted when "Select All" is picked; the popup window listens and selects its files. */
export const POPUP_SELECT_ALL_EVENT = "popup-select-all";

/**
 * Emitted by the backend when the context menu window opens/closes.
 * The popup listens and suppresses its blur/inactivity auto-close
 * while the menu is open.
 */
export const CONTEXT_MENU_OPENED_EVENT = "contextmenu-opened";
export const CONTEXT_MENU_CLOSED_EVENT = "contextmenu-closed";

function MenuItem({
  icon,
  label,
  disabled,
  onClick,
}: {
  icon: ReactNode;
  label: string;
  disabled?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      disabled={disabled}
      onClick={onClick}
      className="flex h-9 w-full items-center gap-2 rounded px-2 text-left text-xs text-primary transition-colors hover:bg-secondary disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:bg-transparent [&>svg]:h-4 [&>svg]:w-4"
    >
      {icon}
      <span className="truncate">{label}</span>
    </button>
  );
}

const ContextMenuWindow: React.FC = () => {
  const { t } = useTranslation();
  const [selectedIds, setSelectedIds] = useState<number[]>([]);

  useEffect(() => {
    invoke<number[]>("get_context_menu_selection")
      .then(setSelectedIds)
      .catch(() => setSelectedIds([]));
  }, []);

  const closeMenu = useCallback(() => {
    invoke("close_context_menu_window").catch(() => {});
  }, []);

  const handleSelectAll = useCallback(() => {
    emit(POPUP_SELECT_ALL_EVENT).catch(() => {});
    closeMenu();
  }, [closeMenu]);

  const handleRemoveSelected = useCallback(() => {
    if (selectedIds.length === 0) return;
    invoke("remove_files", { fileIds: selectedIds })
      .catch((error) => console.error("Failed to remove selected files:", error))
      .finally(closeMenu);
  }, [selectedIds, closeMenu]);

  const handleClearAll = useCallback(() => {
    invoke("clear_files")
      .catch((error) => console.error("Failed to clear files:", error))
      .finally(closeMenu);
  }, [closeMenu]);

  // NOTE: this window is non-activating (focusable(false)) so clicks never
  // steal focus from the popup underneath. Dismissal is owned elsewhere:
  // item click (here), any mousedown in the popup, popup close, or a new
  // right-click repositioning the menu.
  return (
    <div
      className="fixed inset-0 flex flex-col overflow-hidden rounded-md border border-border bg-background p-1"
      onContextMenu={(event) => event.preventDefault()}
    >
      <MenuItem icon={<Check />} label={t("contextMenu.selectAll")} onClick={handleSelectAll} />
      <MenuItem
        icon={<Trash2 />}
        label={t("contextMenu.removeSelected")}
        disabled={selectedIds.length === 0}
        onClick={handleRemoveSelected}
      />
      <div className="mx-1 my-1 h-px shrink-0 bg-border" />
      <MenuItem icon={<X />} label={t("contextMenu.clearAll")} onClick={handleClearAll} />
    </div>
  );
};

export default ContextMenuWindow;
