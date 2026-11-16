import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useTranslation } from 'react-i18next';
import { ExternalLink } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useTheme } from '@/components/theme-provider';
import { closeCurrentWindow } from '@/lib/windowUtils';

interface Item {
  name: string;
  text: string;
}

/** Pages the footer links to; the backend maps each to its URL. */
type ProjectLink = 'donate' | 'featureRequest' | 'reportProblem' | 'changelog';
const LINKS: ProjectLink[] = ['donate', 'featureRequest', 'reportProblem', 'changelog'];

/** "3.2.0" from the backend's `?release=` reads as "3.2" in the title. */
const releaseName = (): string => {
  const release = new URLSearchParams(window.location.search).get('release') ?? '';
  return release.split('.').slice(0, 2).join('.');
};

/**
 * True once Mica has been applied to this window (`<html data-mica>`, set by
 * useWindowStyle). It stays true if the user turns Mica off from here, so
 * the section explaining it doesn't vanish. On Windows 10 it never applies.
 */
function useMicaApplied(): boolean {
  const root = document.documentElement;
  const [applied, setApplied] = useState(() => 'mica' in root.dataset);
  useEffect(() => {
    const observer = new MutationObserver(() => setApplied((was) => was || 'mica' in root.dataset));
    observer.observe(root, { attributes: true, attributeFilter: ['data-mica'] });
    return () => observer.disconnect();
  }, [root]);
  return applied;
}

/**
 * Shown once, on the first start after updating from an older version. The
 * backend decides when (`needs_whats_new`) and records the version as seen
 * when this window closes.
 */
export default function WhatsNew() {
  const { t } = useTranslation();
  const { theme } = useTheme();
  const mainRef = useRef<HTMLElement>(null);
  const micaApplied = useMicaApplied();
  // A link or Settings that couldn't be opened, shown above the links.
  const [failure, setFailure] = useState<string | null>(null);
  const title = t('whatsNew.title', { version: releaseName() });

  useEffect(() => {
    // Start in the text: the arrow keys scroll it, and a screen reader
    // announces it by the title.
    mainRef.current?.focus();
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        closeCurrentWindow();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, []);

  // The taskbar and Alt+Tab show the window title in the user's language.
  useEffect(() => {
    getCurrentWindow().setTitle(title).catch(() => {});
  }, [title]);

  const items = (key: string) => t(key, { returnObjects: true }) as Item[];
  const settingPath = (...keys: string[]) => [t('settings.title'), ...keys.map((key) => t(key))].join(' › ');

  const openMicaSetting = () => {
    setFailure(null);
    invoke('open_settings_window', { theme, field: 'mica' }).catch((error) => {
      console.error(error);
      setFailure(t('whatsNew.settingFailed'));
    });
  };
  const openLink = (link: ProjectLink) => {
    setFailure(null);
    invoke('open_project_link', { link }).catch((error) => {
      console.error(error);
      setFailure(t('whatsNew.linkFailed'));
    });
  };

  const sections: { id: string; title: string; items: Item[] }[] = [
    {
      id: 'changes',
      title: t('whatsNew.changesTitle'),
      items: [
        ...items('whatsNew.changes'),
        {
          name: t('whatsNew.shakeName'),
          text: t('whatsNew.shakeText', {
            path: settingPath('settings.mouse.title', 'settings.mouse.blocklistTitle'),
          }),
        },
      ],
    },
    { id: 'shelves', title: t('whatsNew.shelvesTitle'), items: items('whatsNew.shelves') },
    { id: 'items', title: t('whatsNew.itemsTitle'), items: items('whatsNew.items') },
    { id: 'fixes', title: t('whatsNew.fixesTitle'), items: items('whatsNew.fixes') },
  ];
  if (micaApplied) {
    sections.unshift({
      id: 'setting',
      title: t('whatsNew.settingTitle'),
      items: [
        {
          name: t('whatsNew.micaName'),
          text: t('whatsNew.micaText', {
            path: settingPath('settings.appearance.title', 'settings.appearance.mica'),
          }),
        },
      ],
    });
  }

  return (
    <div className="flex h-full flex-col bg-background text-foreground">
      {/* Stays put while the text scrolls, so the window can always be dragged. */}
      <header className="px-6 pt-6 pb-3" data-tauri-drag-region>
        <h1 id="whats-new-title" className="text-xl font-semibold" data-tauri-drag-region>
          {title}
        </h1>
        <p className="mt-1 text-sm text-foreground/80" data-tauri-drag-region>
          {t('whatsNew.description')}
        </p>
      </header>
      {/* Focusable, so the text can be scrolled from the keyboard. */}
      <main
        ref={mainRef}
        tabIndex={0}
        aria-labelledby="whats-new-title"
        className="flex-1 space-y-4 overflow-y-auto px-6 pb-3 text-[13px] outline-none focus-visible:ring-1 focus-visible:ring-inset focus-visible:ring-ring/50"
      >
        {sections.map((section) => (
          <section key={section.id} aria-labelledby={`whats-new-${section.id}`}>
            <h2 id={`whats-new-${section.id}`} className="mb-1.5 text-sm font-medium">
              {section.title}
            </h2>
            <ul className="space-y-1.5 rounded-md bg-muted p-3">
              {section.items.map((item) => (
                <li key={item.name}>
                  <strong className="font-medium">{item.name}</strong>
                  {/* Full foreground: muted text on the muted panel is too faint. */}
                  <span> {item.text}</span>
                </li>
              ))}
            </ul>
            {section.id === 'setting' && (
              <Button variant="outline" size="sm" className="mt-2" onClick={openMicaSetting}>
                {t('whatsNew.openSetting')}
              </Button>
            )}
          </section>
        ))}
      </main>
      <footer className="space-y-2 border-t border-border/60 bg-muted px-6 pt-3 pb-2">
        {/* The default 1px ring is close to the button's own colour. */}
        <Button
          className="w-full focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-muted"
          onClick={closeCurrentWindow}
        >
          {t('whatsNew.dismiss')}
        </Button>
        <nav aria-label={t('whatsNew.linksLabel')}>
          <ul className="flex flex-wrap justify-center gap-x-1 text-xs">
            {LINKS.map((link) => (
              <li key={link}>
                <button
                  type="button"
                  onClick={() => openLink(link)}
                  className="inline-flex items-center gap-1 rounded px-1.5 py-1 text-foreground/80 hover:text-foreground hover:underline focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                >
                  {t(`whatsNew.${link}`)}
                  <ExternalLink className="h-3 w-3 rtl:-scale-x-100" aria-hidden />
                </button>
              </li>
            ))}
          </ul>
        </nav>
        <p role="status" className="text-center text-xs text-destructive">
          {failure}
        </p>
      </footer>
    </div>
  );
}
