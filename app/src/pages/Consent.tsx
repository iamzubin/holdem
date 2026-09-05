import React, { useEffect, useRef } from 'react';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../components/ui/card';
import { invoke } from '@tauri-apps/api/core';
import { closeCurrentWindow } from '@/lib/windowUtils';
import { useTranslation } from 'react-i18next';

export default function Consent() {
  const { t } = useTranslation();
  const choiceMadeRef = useRef(false);

  // Handle window close events. `.close()` fires `beforeunload` (unlike the
  // old `hide()`), so skip the default decline once the user has chosen —
  // otherwise Accept would be overwritten by Decline on close.
  useEffect(() => {
    const handleBeforeUnload = () => {
      if (choiceMadeRef.current) return;
      // If user closes window without making a choice, decline by default
      invoke('decline_analytics_consent').catch((error) => {
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
      await invoke('accept_analytics_consent');
      // Add a small delay before closing to ensure the event is sent
      setTimeout(() => {
        closeCurrentWindow();
      }, 100);
    } catch (error) {
      console.error('Failed to accept analytics consent:', error);
      closeCurrentWindow();
    }
  };

  const handleDecline = async () => {
    choiceMadeRef.current = true;
    try {
      await invoke('decline_analytics_consent');
      // Add a small delay before closing to ensure the event is sent
      setTimeout(() => {
        closeCurrentWindow();
      }, 100);
    } catch (error) {
      console.error('Failed to decline analytics consent:', error);
      closeCurrentWindow();
    }
  };

  return (
    <div className="h-full bg-background flex items-center justify-center p-4 " data-tauri-drag-region>
      <Card className="w-full max-w-md" data-tauri-drag-region>
        <CardHeader className="text-center" data-tauri-drag-region>
          <CardTitle className="text-xl">{t("consent.title")}</CardTitle>
          <CardDescription data-tauri-drag-region>
            {t("consent.description")}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6" data-tauri-drag-region>
          <div className="text-sm text-muted-foreground space-y-3">
            <div>
              <p className="font-medium mb-2">{t("consent.collectTitle")}</p>
              <ul className="list-disc list-inside space-y-1 ms-4">
                {(t("consent.collectItems", { returnObjects: true }) as string[]).map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
            </div>
            <div>
              <p className="font-medium mb-2">{t("consent.notCollectTitle")}</p>
              <ul className="list-disc list-inside space-y-1 ms-4">
                {(t("consent.notCollectItems", { returnObjects: true }) as string[]).map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
            </div>
          </div>
          <div className="flex gap-3 pt-4">
            <Button 
              variant="outline" 
              onClick={handleDecline}
              className="flex-1"
            >
              {t("consent.decline")}
            </Button>
            <Button 
              onClick={handleAccept}
              className="flex-1"
            >
              {t("consent.accept")}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
