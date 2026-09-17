import { NumberField, SectionTitle, SettingRow } from '@/components/settings-section';
import { useTheme } from '@/components/theme-provider';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { supportedLanguages } from '@/i18n';
import { closeCurrentWindow } from '@/lib/windowUtils';
import { invoke } from '@tauri-apps/api/core';
import { Keyboard, Monitor, Moon, Palette, Settings, Sun, X } from 'lucide-react';
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

const defaults: AppConfig = {
  mouse_monitor: {
    required_shakes: 5,
    shake_time_limit: 1500,
    shake_threshold: 100,
    window_close_delay: 3000,
    whitelist: ['explorer.exe'],
  },
  autostart: false,
  hotkey: '',
  analytics_enabled: false,
  analytics_uuid: '',
};

const MODIFIER_KEYS = ['Control', 'Alt', 'Shift', 'Meta'];

const buildHotkey = (event: KeyboardEvent): string => {
  const parts = [
    event.ctrlKey && 'Ctrl',
    event.altKey && 'Alt',
    event.shiftKey && 'Shift',
    event.metaKey && 'Meta',
  ].filter(Boolean) as string[];
  if (!MODIFIER_KEYS.includes(event.key)) {
    parts.push(
      event.code.startsWith('Key')
        ? event.code.slice(3)
        : event.code.startsWith('Digit')
          ? event.code.slice(5)
          : event.key,
    );
  }
  return parts.join('+');
};

export default function SettingsPage() {
  const { t, i18n } = useTranslation();
  const { theme, setTheme } = useTheme();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [saving, setSaving] = useState(false);
  const [listening, setListening] = useState(false);
  const [currentHotkey, setCurrentHotkey] = useState('');
  const [newApp, setNewApp] = useState('');
  const hotkeyHandlerRef = useRef<((event: KeyboardEvent) => void) | null>(null);

  useEffect(() => {
    invoke<AppConfig>('get_config')
      .then(setConfig)
      .catch((error) => {
        console.error('Failed to load config:', error);
        setConfig(defaults);
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
    try {
      await invoke('save_config', { newConfig: config });
      await invoke('set_autostart', { enabled: config.autostart });
      await invoke('register_hotkey', { shortcutStr: config.hotkey });
      await invoke(config.analytics_enabled ? 'accept_analytics_consent' : 'decline_analytics_consent');
      await closeCurrentWindow();
    } catch (error) {
      console.error('Failed to save settings:', error);
    } finally {
      setSaving(false);
    }
  };

  const stopHotkeyListening = useCallback(() => {
    setListening(false);
    if (hotkeyHandlerRef.current) {
      window.removeEventListener('keydown', hotkeyHandlerRef.current, true);
      hotkeyHandlerRef.current = null;
    }
  }, []);

  const startHotkeyListening = () => {
    setListening(true);
    const handler = (event: KeyboardEvent) => {
      event.preventDefault();
      const value = buildHotkey(event);
      setCurrentHotkey(value);
      if (!MODIFIER_KEYS.includes(event.key)) {
        setConfig((current) => (current ? { ...current, hotkey: value } : current));
        stopHotkeyListening();
      }
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
              icon={<Palette />}
              title={t('settings.language.title')}
              description={t('settings.language.description')}
            />
            <div className="rounded-lg border bg-card p-4">
              <Label htmlFor="language">{t('settings.language.label')}</Label>
              <select
                id="language"
                className="mt-2 h-9 w-full rounded-md border border-input bg-background px-3 text-sm"
                value={i18n.language}
                onChange={(event) => i18n.changeLanguage(event.target.value)}
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
              <Label>{t('settings.shortcuts.showHotkey')}</Label>
              <div className="mt-2 flex gap-2">
                <div className="flex h-9 min-w-0 flex-1 items-center rounded-md border bg-background px-3 font-mono text-sm">
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
            </div>
          </section>

          <section className="space-y-3">
            <SectionTitle
              icon={<Monitor />}
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
        <Button className="w-full" onClick={saveConfig} disabled={saving}>
          {saving ? t('settings.footer.saving') : t('settings.footer.save')}
        </Button>
      </footer>
    </div>
  );
}
