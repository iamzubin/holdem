import { NumberField, SectionTitle, SettingRow } from '@/components/settings-section';
import { useTheme } from '@/components/theme-provider';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { setLanguage, supportedLanguages } from '@/i18n';
import { closeCurrentWindow } from '@/lib/windowUtils';
import { invoke } from '@tauri-apps/api/core';
import { Keyboard, Languages, Monitor, Moon, Mouse, Settings, Sun, X } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

interface MouseMonitorConfig {
  required_shakes: number;
  shake_time_limit: number;
  shake_threshold: number;
  window_close_delay: number;
  whitelist: string[];
}

interface AppConfig {
  mouse_monitor: MouseMonitorConfig;
  autostart: boolean;
  hotkey: string;
  analytics_enabled: boolean;
  analytics_uuid: string;
}

const MODIFIER_KEYS = ['Control', 'Alt', 'Shift', 'Meta'];

// Matches the backend rule: a key with no modifier would be swallowed in
// every app, so only F1-F24 may stand alone.
const FUNCTION_KEY = /^F([1-9]|1[0-9]|2[0-4])$/;

const buildHotkey = (event: KeyboardEvent): string => {
  const parts = [
    event.ctrlKey && 'Ctrl',
    event.altKey && 'Alt',
    event.shiftKey && 'Shift',
    event.metaKey && 'Meta',
  ].filter(Boolean) as string[];
  if (!MODIFIER_KEYS.includes(event.key)) {
    // Use the physical key code (Space, ArrowUp, F5, Comma, Numpad1, ...) so
    // the backend can map it; `event.key` varies with Shift/layout and is " "
    // for Space.
    parts.push(
      event.code.startsWith('Key')
        ? event.code.slice(3)
        : event.code.startsWith('Digit')
          ? event.code.slice(5)
          : event.code,
    );
  }
  return parts.join('+');
};

export default function SettingsPage() {
  const { t, i18n } = useTranslation();
  const { theme, setTheme } = useTheme();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [listening, setListening] = useState(false);
  const [currentHotkey, setCurrentHotkey] = useState('');
  const [hotkeyHint, setHotkeyHint] = useState(false);
  const [newApp, setNewApp] = useState('');
  const hotkeyHandlerRef = useRef<((event: KeyboardEvent) => void) | null>(null);
  // Hotkey as saved on disk, so an unchanged hotkey never blocks saving
  // (older versions saved strings the backend may not accept) and a failed
  // save can restore it.
  const savedHotkeyRef = useRef('');

  useEffect(() => {
    invoke<AppConfig>('get_config')
      .then((loaded) => {
        savedHotkeyRef.current = loaded.hotkey;
        setConfig(loaded);
      })
      .catch((error) => {
        // Don't fall back to defaults: saving them would overwrite the
        // user's real config.
        console.error('Failed to load config:', error);
        setLoadError(String(error));
      });
  }, []);

  useEffect(
    () => () => {
      if (hotkeyHandlerRef.current) {
        window.removeEventListener('keydown', hotkeyHandlerRef.current, true);
      }
    },
    [],
  );

  const updateMouse = useCallback((key: keyof MouseMonitorConfig, value: number | string[]) => {
    setConfig((current) =>
      current ? { ...current, mouse_monitor: { ...current.mouse_monitor, [key]: value } } : current,
    );
  }, []);

  const saveConfig = async () => {
    if (!config) return;
    setSaving(true);
    setSaveError(null);
    const hotkeyChanged = config.hotkey !== savedHotkeyRef.current;
    let hotkeyRegistered = false;
    try {
      // Register a changed hotkey first: it validates the key and fails
      // without touching the saved config or the currently active hotkey.
      if (hotkeyChanged) {
        await invoke('register_hotkey', { shortcutStr: config.hotkey });
        hotkeyRegistered = true;
      }
      // Also applies the analytics consent to the running app.
      await invoke('save_config', { newConfig: config });
      savedHotkeyRef.current = config.hotkey;
      hotkeyRegistered = false;
      try {
        await invoke('set_autostart', { enabled: config.autostart });
      } catch (error) {
        // The config is already saved; only the startup entry failed.
        console.error('Failed to update autostart:', error);
        setSaveError(`${t('settings.footer.autostartFailed')}: ${String(error)}`);
        return;
      }
      // An unchanged hotkey can still be inactive if its registration failed
      // at startup (another app held the combo, or an old saved string).
      // Retry it after saving so a bad hotkey never blocks other settings.
      if (!hotkeyChanged && config.hotkey.trim() !== '') {
        const active = await invoke<boolean>('is_hotkey_active', { shortcutStr: config.hotkey });
        if (!active) {
          try {
            await invoke('register_hotkey', { shortcutStr: config.hotkey });
          } catch (error) {
            console.error('Failed to register saved hotkey:', error);
            setSaveError(`${t('settings.footer.hotkeyFailed')}: ${String(error)}`);
            return;
          }
        }
      }
      await closeCurrentWindow();
    } catch (error) {
      console.error('Failed to save settings:', error);
      setSaveError(`${t('settings.footer.saveFailed')}: ${String(error)}`);
      if (hotkeyRegistered) {
        // The config was not saved: go back to the hotkey that is on disk.
        await invoke('register_hotkey', { shortcutStr: savedHotkeyRef.current }).catch((e) =>
          console.error('Failed to restore previous hotkey:', e),
        );
      }
    } finally {
      setSaving(false);
    }
  };

  const stopHotkeyListening = useCallback(() => {
    setListening(false);
    setHotkeyHint(false);
    if (hotkeyHandlerRef.current) {
      window.removeEventListener('keydown', hotkeyHandlerRef.current, true);
      hotkeyHandlerRef.current = null;
    }
  }, []);

  const startHotkeyListening = () => {
    setListening(true);
    setCurrentHotkey('');
    setHotkeyHint(false);
    const handler = (event: KeyboardEvent) => {
      event.preventDefault();
      const hasModifier = event.ctrlKey || event.altKey || event.shiftKey || event.metaKey;
      if (event.key === 'Escape' && !hasModifier) {
        // Cancel and keep the current hotkey.
        stopHotkeyListening();
        return;
      }
      const value = buildHotkey(event);
      setCurrentHotkey(value);
      if (MODIFIER_KEYS.includes(event.key)) return;
      if (!hasModifier && !FUNCTION_KEY.test(event.code)) {
        // Keep listening for a combination instead of taking a bare key.
        setHotkeyHint(true);
        return;
      }
      setConfig((current) => (current ? { ...current, hotkey: value } : current));
      stopHotkeyListening();
    };
    hotkeyHandlerRef.current = handler;
    window.addEventListener('keydown', handler, true);
  };

  const addWhitelistApp = () => {
    if (!config) return;
    const item = newApp.trim();
    if (item && !config.mouse_monitor.whitelist.includes(item)) {
      updateMouse('whitelist', [...config.mouse_monitor.whitelist, item]);
    }
    setNewApp('');
  };

  if (loadError) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
        <p role="alert" className="text-sm text-destructive">
          {t('settings.loadFailed')}: {loadError}
        </p>
        <Button variant="outline" size="sm" onClick={closeCurrentWindow}>
          {t('common.close')}
        </Button>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
        {t('settings.loading')}
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col bg-background text-foreground">
      <header
        className="flex min-h-14 items-center justify-between border-b border-border/60 px-4"
        data-tauri-drag-region
      >
        <div className="flex items-center gap-3" data-tauri-drag-region>
          <div className="rounded-lg bg-primary/10 p-2">
            <Settings className="h-4 w-4 text-primary" />
          </div>
          <div>
            <h1 className="text-sm font-semibold">{t('settings.title')}</h1>
            <p className="text-xs text-muted-foreground">{t('settings.subtitle')}</p>
          </div>
        </div>
        <Button variant="ghost" size="icon" aria-label={t('common.close')} onClick={closeCurrentWindow}>
          <X className="h-4 w-4" />
        </Button>
      </header>

      <main className="flex-1 overflow-y-auto px-5 py-5">
        <div className="mx-auto max-w-xl space-y-6">
          <section className="space-y-3">
            <SectionTitle
              icon={<Monitor />}
              title={t('settings.general.title')}
              description={t('settings.general.description')}
            />
            <div className="divide-y rounded-lg border bg-card">
              <SettingRow
                label={t('settings.general.startup')}
                description={t('settings.general.startupDesc')}
              >
                <Switch
                  id="startup"
                  label={t('settings.general.startup')}
                  checked={config.autostart}
                  onChange={() => setConfig({ ...config, autostart: !config.autostart })}
                />
              </SettingRow>
              <SettingRow
                label={t('settings.general.analytics')}
                description={t('settings.general.analyticsDesc')}
              >
                <Switch
                  id="analytics"
                  label={t('settings.general.analytics')}
                  checked={config.analytics_enabled}
                  onChange={() =>
                    setConfig({ ...config, analytics_enabled: !config.analytics_enabled })
                  }
                />
              </SettingRow>
            </div>
          </section>

          <section className="space-y-3">
            <SectionTitle
              icon={<Languages />}
              title={t('settings.language.title')}
              description={t('settings.language.description')}
            />
            <div className="rounded-lg border bg-card p-4">
              <Label htmlFor="language">{t('settings.language.label')}</Label>
              <select
                id="language"
                className="mt-2 h-9 w-full rounded-md border border-input bg-background px-3 text-sm"
                value={i18n.resolvedLanguage ?? i18n.language}
                onChange={(event) => setLanguage(event.target.value)}
              >
                {supportedLanguages.map((language) => (
                  <option key={language.code} value={language.code}>
                    {language.nativeLabel}
                  </option>
                ))}
              </select>
            </div>
          </section>

          <section className="space-y-3">
            <SectionTitle
              icon={<Keyboard />}
              title={t('settings.shortcuts.title')}
              description={t('settings.shortcuts.description')}
            />
            <div className="rounded-lg border bg-card p-4">
              <Label id="hotkey-label">{t('settings.shortcuts.showHotkey')}</Label>
              <div className="mt-2 flex gap-2">
                <div
                  role="status"
                  aria-labelledby="hotkey-label"
                  className="flex h-9 min-w-0 flex-1 items-center rounded-md border bg-background px-3 font-mono text-sm"
                >
                  {listening
                    ? currentHotkey || t('settings.shortcuts.pressKeys')
                    : config.hotkey || t('settings.shortcuts.noneSet')}
                </div>
                <Button
                  size="sm"
                  variant={listening ? 'destructive' : 'default'}
                  onClick={() => (listening ? stopHotkeyListening() : startHotkeyListening())}
                >
                  {listening ? t('settings.shortcuts.stop') : t('settings.shortcuts.set')}
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => setConfig({ ...config, hotkey: '' })}
                >
                  {t('settings.shortcuts.clear')}
                </Button>
              </div>
              {listening && (
                <p
                  aria-live="polite"
                  className={
                    hotkeyHint ? 'mt-2 text-xs text-destructive' : 'mt-2 text-xs text-muted-foreground'
                  }
                >
                  {t(hotkeyHint ? 'settings.shortcuts.needsModifier' : 'settings.shortcuts.escToCancel')}
                </p>
              )}
            </div>
          </section>

          <section className="space-y-3">
            <SectionTitle
              icon={<Mouse />}
              title={t('settings.mouse.title')}
              description={t('settings.mouse.description')}
            />
            <div className="rounded-lg border bg-card p-4">
              <div className="grid gap-4 sm:grid-cols-2">
                <NumberField
                  id="required_shakes"
                  label={t('settings.mouse.requiredShakes')}
                  value={config.mouse_monitor.required_shakes}
                  min={1}
                  max={20}
                  onChange={(value) => updateMouse('required_shakes', value)}
                />
                <NumberField
                  id="shake_threshold"
                  label={t('settings.mouse.shakeThreshold')}
                  value={config.mouse_monitor.shake_threshold}
                  min={1}
                  max={1000}
                  onChange={(value) => updateMouse('shake_threshold', value)}
                />
                <NumberField
                  id="shake_time_limit"
                  label={t('settings.mouse.timeLimit')}
                  value={config.mouse_monitor.shake_time_limit}
                  min={100}
                  max={10000}
                  onChange={(value) => updateMouse('shake_time_limit', value)}
                />
                <NumberField
                  id="window_close_delay"
                  label={t('settings.mouse.closeDelay')}
                  value={config.mouse_monitor.window_close_delay}
                  min={0}
                  max={30000}
                  onChange={(value) => updateMouse('window_close_delay', value)}
                />
              </div>
              <div className="mt-5 space-y-2 border-t pt-4">
                <Label htmlFor="new-app">{t('settings.mouse.whitelistTitle')}</Label>
                <p className="text-xs text-muted-foreground">{t('settings.mouse.whitelistDesc')}</p>
                <div className="flex gap-2">
                  <Input
                    id="new-app"
                    value={newApp}
                    placeholder={t('settings.mouse.addPlaceholder')}
                    onChange={(event) => setNewApp(event.target.value)}
                    onKeyDown={(event) => {
                      if (event.key === 'Enter') addWhitelistApp();
                    }}
                  />
                  <Button size="sm" onClick={addWhitelistApp}>
                    {t('settings.mouse.add')}
                  </Button>
                </div>
                <div className="flex flex-wrap gap-2">
                  {config.mouse_monitor.whitelist.map((app) => (
                    <button
                      key={app}
                      type="button"
                      aria-label={t('settings.mouse.removeApp', { app })}
                      className="rounded-md bg-muted px-2 py-1 font-mono text-xs hover:bg-destructive/10"
                      onClick={() =>
                        updateMouse(
                          'whitelist',
                          config.mouse_monitor.whitelist.filter((item) => item !== app),
                        )
                      }
                    >
                      {app} <span aria-hidden>×</span>
                    </button>
                  ))}
                </div>
              </div>
            </div>
          </section>

          <section className="space-y-3">
            <SectionTitle
              icon={theme === 'dark' ? <Moon /> : <Sun />}
              title={t('settings.appearance.title')}
              description={t('settings.appearance.description')}
            />
            <div className="flex gap-2 rounded-lg border bg-card p-2">
              {(['system', 'light', 'dark'] as const).map((option) => (
                <Button
                  key={option}
                  size="sm"
                  variant={theme === option ? 'default' : 'ghost'}
                  aria-pressed={theme === option}
                  className="flex-1 capitalize"
                  onClick={() => setTheme(option)}
                >
                  {t(`settings.appearance.${option}`)}
                </Button>
              ))}
            </div>
          </section>
          <p className="text-center text-xs text-muted-foreground">
            {t('settings.dragDrop.dragToCopy')} · {t('settings.dragDrop.shiftToMove')}
          </p>
        </div>
      </main>
      <footer className="border-t bg-background/95 p-4">
        {saveError && (
          <p role="alert" className="mb-2 text-xs text-destructive">
            {saveError}
          </p>
        )}
        <Button className="w-full" onClick={saveConfig} disabled={saving}>
          {saving ? t('settings.footer.saving') : t('settings.footer.save')}
        </Button>
      </footer>
    </div>
  );
}
