import { invoke } from '@tauri-apps/api/core'
import { Keyboard, Monitor, Moon, Palette, Settings, Sun, X } from 'lucide-react'
import { useEffect, useState } from 'react'
import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { supportedLanguages } from '@/i18n'
import { useTheme } from '@/components/theme-provider'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

interface MouseMonitorConfig {
  required_shakes: number
  shake_time_limit: number
  shake_threshold: number
  window_close_delay: number
  whitelist: string[]
}

interface AppConfig {
  mouse_monitor: MouseMonitorConfig
  autostart: boolean
  hotkey: string
  analytics_enabled: boolean
  analytics_uuid: string
}

const defaults: AppConfig = {
  mouse_monitor: { required_shakes: 5, shake_time_limit: 1500, shake_threshold: 100, window_close_delay: 3000, whitelist: ['explorer.exe'] },
  autostart: false,
  hotkey: '',
  analytics_enabled: false,
  analytics_uuid: '',
}

function Switch({ id, checked, onChange }: { id: string; checked: boolean; onChange: () => void }) {
  return <button id={id} type="button" role="switch" aria-checked={checked} onClick={onChange}
    className={`relative inline-flex h-6 w-11 shrink-0 rounded-full border-2 border-transparent transition-colors active:scale-[.97] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${checked ? 'bg-primary' : 'bg-input'}`}>
    <span className={`pointer-events-none block h-5 w-5 rounded-full bg-background shadow transition-transform duration-150 ease-out ${checked ? 'translate-x-5 rtl:-translate-x-5' : ''}`} />
  </button>
}

export default function SettingsPage() {
  const { t, i18n } = useTranslation()
  const { theme, setTheme } = useTheme()
  const [config, setConfig] = useState<AppConfig | null>(null)
  const [saving, setSaving] = useState(false)
  const [listening, setListening] = useState(false)
  const [currentHotkey, setCurrentHotkey] = useState('')
  const [newApp, setNewApp] = useState('')

  useEffect(() => {
    invoke<AppConfig>('get_config').then(setConfig).catch((error) => {
      console.error('Failed to load config:', error)
      setConfig(defaults)
    })
  }, [])

  const updateMouse = (key: keyof MouseMonitorConfig, value: number | string[]) => {
    setConfig((current) => current ? { ...current, mouse_monitor: { ...current.mouse_monitor, [key]: value } } : current)
  }

  const saveConfig = async () => {
    if (!config) return
    setSaving(true)
    try {
      await invoke('save_config', { newConfig: config })
      await invoke('set_autostart', { enabled: config.autostart })
      await invoke('register_hotkey', { shortcutStr: config.hotkey })
      await invoke(config.analytics_enabled ? 'accept_analytics_consent' : 'decline_analytics_consent')
      await invoke('close_settings_window')
    } catch (error) {
      console.error('Failed to save settings:', error)
    } finally {
      setSaving(false)
    }
  }

  const buildHotkey = (event: KeyboardEvent) => {
    const parts = [event.ctrlKey && 'Ctrl', event.altKey && 'Alt', event.shiftKey && 'Shift', event.metaKey && 'Meta'].filter(Boolean)
    const modifier = ['Control', 'Alt', 'Shift', 'Meta'].includes(event.key)
    if (!modifier) parts.push(event.code.startsWith('Key') ? event.code.slice(3) : event.code.startsWith('Digit') ? event.code.slice(5) : event.key)
    return parts.join('+')
  }

  const startHotkey = () => {
    setListening(true)
    const handler = (event: KeyboardEvent) => {
      event.preventDefault()
      const value = buildHotkey(event)
      setCurrentHotkey(value)
      if (!['Control', 'Alt', 'Shift', 'Meta'].includes(event.key)) {
        setConfig((current) => current ? { ...current, hotkey: value } : current)
        setListening(false)
        window.removeEventListener('keydown', handler, true)
      }
    }
    window.addEventListener('keydown', handler, true)
  }

  if (!config) return <div className="flex h-full items-center justify-center text-sm text-muted-foreground">{t('settings.loading')}</div>

  const field = (id: string, label: string, value: number, min: number, max: number) => (
    <div className="space-y-1.5">
      <Label htmlFor={id}>{label}</Label>
      <Input id={id} type="number" min={min} max={max} value={value}
        onChange={(event) => updateMouse(id as keyof MouseMonitorConfig, Math.min(max, Math.max(min, Number(event.target.value) || min)))} />
    </div>
  )

  return <div className="flex h-full flex-col bg-background text-foreground">
    <header className="flex min-h-14 items-center justify-between border-b border-border/60 px-4" data-tauri-drag-region>
      <div className="flex items-center gap-3" data-tauri-drag-region>
        <div className="rounded-lg bg-primary/10 p-2"><Settings className="h-4 w-4 text-primary" /></div>
        <div><h1 className="text-sm font-semibold">{t('settings.title')}</h1><p className="text-xs text-muted-foreground">{t('settings.subtitle')}</p></div>
      </div>
      <Button variant="ghost" size="icon" aria-label={t('common.close')} onClick={() => invoke('close_settings_window')}><X className="h-4 w-4" /></Button>
    </header>

    <main className="flex-1 overflow-y-auto px-5 py-5">
      <div className="mx-auto max-w-xl space-y-6">
        <section className="space-y-3"><SectionTitle icon={<Monitor />} title={t('settings.general.title')} description={t('settings.general.description')} />
          <div className="divide-y rounded-lg border bg-card">
            <SettingRow label={t('settings.general.startup')} description={t('settings.general.startupDesc')}><Switch id="startup" checked={config.autostart} onChange={() => setConfig({ ...config, autostart: !config.autostart })} /></SettingRow>
            <SettingRow label={t('settings.general.analytics')} description={t('settings.general.analyticsDesc')}><Switch id="analytics" checked={config.analytics_enabled} onChange={() => setConfig({ ...config, analytics_enabled: !config.analytics_enabled })} /></SettingRow>
          </div>
        </section>

        <section className="space-y-3"><SectionTitle icon={<Palette />} title={t('settings.language.title')} description={t('settings.language.description')} />
          <div className="rounded-lg border bg-card p-4"><Label htmlFor="language">{t('settings.language.label')}</Label><select id="language" className="mt-2 h-9 w-full rounded-md border border-input bg-background px-3 text-sm" value={i18n.language} onChange={(event) => i18n.changeLanguage(event.target.value)}>{supportedLanguages.map((language) => <option key={language.code} value={language.code}>{language.nativeLabel}</option>)}</select></div>
        </section>

        <section className="space-y-3"><SectionTitle icon={<Keyboard />} title={t('settings.shortcuts.title')} description={t('settings.shortcuts.description')} />
          <div className="rounded-lg border bg-card p-4"><Label>{t('settings.shortcuts.showHotkey')}</Label><div className="mt-2 flex gap-2"><div className="flex h-9 min-w-0 flex-1 items-center rounded-md border bg-background px-3 font-mono text-sm">{listening ? currentHotkey || t('settings.shortcuts.pressKeys') : config.hotkey || t('settings.shortcuts.noneSet')}</div><Button size="sm" variant={listening ? 'destructive' : 'default'} onClick={() => listening ? setListening(false) : startHotkey()}>{listening ? t('settings.shortcuts.stop') : t('settings.shortcuts.set')}</Button><Button size="sm" variant="outline" onClick={() => setConfig({ ...config, hotkey: '' })}>{t('settings.shortcuts.clear')}</Button></div></div>
        </section>

        <section className="space-y-3"><SectionTitle icon={<Monitor />} title={t('settings.mouse.title')} description={t('settings.mouse.description')} />
          <div className="rounded-lg border bg-card p-4"><div className="grid gap-4 sm:grid-cols-2">
            {field('required_shakes', t('settings.mouse.requiredShakes'), config.mouse_monitor.required_shakes, 1, 20)}
            {field('shake_threshold', t('settings.mouse.shakeThreshold'), config.mouse_monitor.shake_threshold, 1, 1000)}
            {field('shake_time_limit', t('settings.mouse.timeLimit'), config.mouse_monitor.shake_time_limit, 100, 10000)}
            {field('window_close_delay', t('settings.mouse.closeDelay'), config.mouse_monitor.window_close_delay, 0, 30000)}
          </div><div className="mt-5 space-y-2 border-t pt-4"><Label htmlFor="new-app">{t('settings.mouse.whitelistTitle')}</Label><p className="text-xs text-muted-foreground">{t('settings.mouse.whitelistDesc')}</p><div className="flex gap-2"><Input id="new-app" value={newApp} placeholder={t('settings.mouse.addPlaceholder')} onChange={(event) => setNewApp(event.target.value)} onKeyDown={(event) => { if (event.key === 'Enter') { const item = newApp.trim(); if (item && !config.mouse_monitor.whitelist.includes(item)) updateMouse('whitelist', [...config.mouse_monitor.whitelist, item]); setNewApp('') } }} /><Button size="sm" onClick={() => { const item = newApp.trim(); if (item && !config.mouse_monitor.whitelist.includes(item)) updateMouse('whitelist', [...config.mouse_monitor.whitelist, item]); setNewApp('') }}>{t('settings.mouse.add')}</Button></div><div className="flex flex-wrap gap-2">{config.mouse_monitor.whitelist.map((app) => <button key={app} type="button" className="rounded-md bg-muted px-2 py-1 font-mono text-xs hover:bg-destructive/10" onClick={() => updateMouse('whitelist', config.mouse_monitor.whitelist.filter((item) => item !== app))}>{app} <span aria-hidden>×</span></button>)}</div></div></div>
        </section>

        <section className="space-y-3"><SectionTitle icon={theme === 'dark' ? <Moon /> : <Sun />} title="Appearance" description="Choose how Holdem looks." /><div className="flex gap-2 rounded-lg border bg-card p-2">{(['system', 'light', 'dark'] as const).map((option) => <Button key={option} size="sm" variant={theme === option ? 'default' : 'ghost'} className="flex-1 capitalize" onClick={() => setTheme(option)}>{option}</Button>)}</div></section>
        <p className="text-center text-xs text-muted-foreground">{t('settings.dragDrop.dragToCopy')} · {t('settings.dragDrop.shiftToMove')}</p>
      </div>
    </main>
    <footer className="border-t bg-background/95 p-4"><Button className="w-full" onClick={saveConfig} disabled={saving}>{saving ? t('settings.footer.saving') : t('settings.footer.save')}</Button></footer>
  </div>
}

function SectionTitle({ icon, title, description }: { icon: ReactNode; title: string; description: string }) {
  return <div className="flex items-start gap-2"><span className="mt-0.5 text-primary [&>svg]:h-4 [&>svg]:w-4">{icon}</span><div><h2 className="text-sm font-semibold">{title}</h2><p className="text-xs text-muted-foreground">{description}</p></div></div>
}

function SettingRow({ label, description, children }: { label: string; description: string; children: ReactNode }) {
  return <div className="flex items-center justify-between gap-4 p-4"><div><p className="text-sm font-medium">{label}</p><p className="text-xs text-muted-foreground">{description}</p></div>{children}</div>
}
