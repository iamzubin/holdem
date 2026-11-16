import { Button } from '@/components/ui/button';
import { FilePreview } from '@/types';
import { invoke } from '@tauri-apps/api/core';
import { File as FileIcon, Folder, X } from 'lucide-react';
import React, { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

/** What the backend sends (src-tauri/src/preview.rs). */
type Preview =
  | { kind: 'image'; mime: string; data: string }
  | { kind: 'text'; text: string; truncated: boolean }
  | { kind: 'folder'; entries: { name: string; folder: boolean }[]; total: number }
  | { kind: 'thumbnail'; data: string };

interface PreviewPaneProps {
  file: FilePreview;
  /** "3 of 12" */
  position: string;
  sizeLabel: string;
  onClose: () => void;
}

/** Space-bar preview over the item list. Arrows step through items, Space or Esc closes. */
export const PreviewPane: React.FC<PreviewPaneProps> = ({ file, position, sizeLabel, onClose }) => {
  const { t } = useTranslation();
  const [preview, setPreview] = useState<Preview | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setPreview(null);
    setFailed(false);
    invoke<Preview>('get_preview', { id: file.id })
      .then((result) => {
        if (!cancelled) setPreview(result);
      })
      .catch((error) => {
        console.error('Preview failed:', error);
        if (!cancelled) setFailed(true);
      });
    return () => {
      cancelled = true;
    };
  }, [file.id]);

  let body: React.ReactNode;
  if (failed) {
    body = <p className="text-xs text-muted-foreground">{t('preview.failed')}</p>;
  } else if (!preview) {
    body = <p className="text-xs text-muted-foreground">{t('preview.loading')}</p>;
  } else if (preview.kind === 'image' || preview.kind === 'thumbnail') {
    const src =
      preview.kind === 'image'
        ? `data:${preview.mime};base64,${preview.data}`
        : `data:image/png;base64,${preview.data}`;
    body = (
      <img
        src={src}
        alt={file.name}
        className="max-h-full max-w-full rounded object-contain"
        draggable={false}
      />
    );
  } else if (preview.kind === 'text') {
    body = (
      <div className="h-full w-full overflow-auto rounded border border-border bg-card p-2">
        <pre className="select-text whitespace-pre-wrap break-words font-mono text-[11px] leading-snug">
          {preview.text}
        </pre>
        {preview.truncated && (
          <p className="mt-2 text-[11px] text-muted-foreground">{t('preview.truncated')}</p>
        )}
      </div>
    );
  } else {
    const more = preview.total - preview.entries.length;
    body = (
      <div className="h-full w-full overflow-auto rounded border border-border bg-card p-2">
        {preview.total === 0 && (
          <p className="text-xs text-muted-foreground">{t('preview.emptyFolder')}</p>
        )}
        <ul className="space-y-0.5">
          {preview.entries.map((entry) => (
            <li key={entry.name} className="flex items-center gap-1.5 text-xs">
              {entry.folder ? (
                <Folder className="h-3.5 w-3.5 shrink-0 text-blue-500" aria-hidden />
              ) : (
                <FileIcon className="h-3.5 w-3.5 shrink-0 text-muted-foreground" aria-hidden />
              )}
              <span className="truncate">{entry.name}</span>
            </li>
          ))}
        </ul>
        {more > 0 && (
          <p className="mt-1 text-[11px] text-muted-foreground">{t('preview.more', { count: more })}</p>
        )}
      </div>
    );
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={t('preview.title', { name: file.name })}
      className="absolute inset-0 z-40 flex flex-col bg-popover p-2 text-popover-foreground"
    >
      <div className="mb-2 flex shrink-0 items-center gap-2">
        <div className="min-w-0 flex-1">
          <p className="truncate text-xs font-semibold" title={file.name}>
            {file.name}
          </p>
          <p className="text-[11px] text-muted-foreground">
            {sizeLabel} · {position} · {t('preview.keys')}
          </p>
        </div>
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          aria-label={t('common.close')}
          onClick={onClose}
          autoFocus
        >
          <X className="h-4 w-4" />
        </Button>
      </div>
      <div className="flex min-h-0 flex-1 items-center justify-center">{body}</div>
    </div>
  );
};
