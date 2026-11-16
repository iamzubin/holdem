import React, { useEffect, useRef, useState } from 'react';
import { Button } from '@/components/ui/button';
import { File as FileIcon, MousePointer2 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { closeCurrentWindow } from '@/lib/windowUtils';
import { useTranslation } from 'react-i18next';

export default function Consent() {
  const { t } = useTranslation();
  const choiceMadeRef = useRef(false);
  const [startOnLogin, setStartOnLogin] = useState(true);
  const [step, setStep] = useState<'welcome' | 'privacy'>('welcome');
  // The hotkey that works right now, if any (new installs get one).
  const [hotkey, setHotkey] = useState<string | null>(null);

  useEffect(() => {
    invoke<{ hotkey: string }>('get_config')
      .then(async ({ hotkey: saved }) => {
        if (saved.trim() === '') return;
        const active = await invoke<boolean>('is_hotkey_active', { shortcutStr: saved });
        if (active) setHotkey(saved);
      })
      .catch(() => {});
  }, []);

  // Handle window close events. `.close()` fires `beforeunload` (unlike the
  // old `hide()`), so skip the default decline once the user has chosen —
  // otherwise Accept would be overwritten by Decline on close.
  useEffect(() => {
    const handleBeforeUnload = () => {
      if (choiceMadeRef.current) return;
      // If user closes window without making a choice, decline by default
      // Closed without choosing: share nothing and don't start on login.
      invoke('decline_analytics_consent', { startOnLogin: false }).catch((error) => {
        console.error('Failed to decline analytics consent during window close:', error);
      });
    };

    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  }, []);

  const handleAccept = async () => {
    choiceMadeRef.current = true;
    try {
      await invoke('accept_analytics_consent', { startOnLogin });
      closeCurrentWindow();
    } catch (error) {
      console.error('Failed to accept analytics consent:', error);
      closeCurrentWindow();
    }
  };

  const handleDecline = async () => {
    choiceMadeRef.current = true;
    try {
      await invoke('decline_analytics_consent', { startOnLogin });
      closeCurrentWindow();
    } catch (error) {
      console.error('Failed to decline analytics consent:', error);
      closeCurrentWindow();
    }
  };

  const list = (key: string) => t(key, { returnObjects: true }) as string[];

  if (step === 'welcome') {
    return (
      <div className="flex h-full flex-col bg-background text-foreground">
        <main className="flex-1 overflow-y-auto px-6 pt-6 pb-3" data-tauri-drag-region>
          <header className="mb-4 text-center" data-tauri-drag-region>
            <h1 className="text-xl font-semibold">{t("welcome.title")}</h1>
            <p className="mt-1 text-sm text-muted-foreground">{t("welcome.description")}</p>
          </header>
          {/* The gesture itself: a file dragged left and right. */}
          <div className="mb-4 flex h-20 items-center justify-center" aria-hidden>
            <div className="relative animate-[holdem-shake_1.6s_ease-in-out_infinite] motion-reduce:animate-none">
              <FileIcon className="h-10 w-10 text-primary" strokeWidth={1.5} />
              <MousePointer2 className="absolute -bottom-2 -end-3 h-5 w-5 fill-foreground text-background" />
            </div>
          </div>
          <section className="rounded-md bg-muted p-4 text-[13px]">
            <h2 className="mb-2 font-medium">{t("welcome.howTitle")}</h2>
            <ol className="list-decimal space-y-1.5 ps-5">
              {list("welcome.how").map((item, index) => (
                // The last way is the hotkey: name it when one works.
                <li key={item}>
                  {index === 2 && hotkey ? t("welcome.hotkeyActive", { hotkey }) : item}
                </li>
              ))}
            </ol>
          </section>
          <p className="mt-3 text-[13px] text-muted-foreground">{t("welcome.pasteTip")}</p>
          <p className="mt-2 text-[13px] text-muted-foreground">{t("settings.dragDrop.dragToCopy")} · {t("settings.dragDrop.shiftToMove")}</p>
        </main>
        <footer className="space-y-3 border-t border-border/60 px-6 py-4">
          <label className="flex items-center gap-2 text-[13px]">
            <input
              type="checkbox"
              className="h-4 w-4 accent-primary"
              checked={startOnLogin}
              onChange={(event) => setStartOnLogin(event.target.checked)}
            />
            {t("consent.startOnLogin")}
          </label>
          <Button className="w-full" onClick={() => setStep('privacy')}>
            {t("welcome.next")}
          </Button>
        </footer>
      </div>
    );
  }

  return (
    // The text scrolls; the two choices stay pinned at the bottom so they are
    // always visible, whatever the language or Windows text size.
    <div className="flex h-full flex-col bg-background text-foreground">
      <main className="flex-1 overflow-y-auto px-6 pt-6 pb-3" data-tauri-drag-region>
        <header className="mb-4 text-center" data-tauri-drag-region>
          <h1 className="text-xl font-semibold">{t("consent.title")}</h1>
          <p className="mt-1 text-sm text-muted-foreground">{t("consent.description")}</p>
        </header>
        <div className="space-y-3 text-[13px] text-muted-foreground">
          <section>
            <h2 className="mb-1.5 font-medium text-foreground">{t("consent.collectTitle")}</h2>
            <ul className="list-disc space-y-1 ps-5">
              {list("consent.collectItems").map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </section>
          <section>
            <h2 className="mb-1.5 font-medium text-foreground">{t("consent.notCollectTitle")}</h2>
            <ul className="list-disc space-y-1 ps-5">
              {list("consent.notCollectItems").map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </section>
          <p>{t("consent.privacyNote")}</p>
          <p>{t("consent.webImages")}</p>
          <p>{t("consent.changeLater")}</p>
        </div>
      </main>
      {/* Equal weight: neither choice is presented as the expected one. */}
      <footer className="flex gap-3 border-t border-border/60 px-6 py-4">
        <Button variant="outline" onClick={handleDecline} className="flex-1">
          {t("consent.decline")}
        </Button>
        <Button variant="outline" onClick={handleAccept} className="flex-1">
          {t("consent.accept")}
        </Button>
      </footer>
    </div>
  );
}
