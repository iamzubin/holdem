import { invoke } from '@tauri-apps/api/core';
import { register, unregisterAll } from '@tauri-apps/plugin-global-shortcut';
import { X, Plus, Trash2, Keyboard, Monitor, Settings, Info } from 'lucide-react';
import { useEffect, useState } from 'react';
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { Label } from "@/components/ui/label"
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/components/ui/card"


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

export default function SettingsPage() {
    const [config, setConfig] = useState<AppConfig | null>(null);
    const [saving, setSaving] = useState(false);
    const [isListening, setIsListening] = useState(false);
    const [currentHotkey, setCurrentHotkey] = useState<string>('');
    const [newWhitelistItem, setNewWhitelistItem] = useState('');

    useEffect(() => {
        loadConfig();

        // Clean up event listeners when component unmounts
        return () => {
            if (isListening) {
                window.removeEventListener('keydown', handleKeyDown);
                window.removeEventListener('keyup', handleKeyUp);
            }
        };
    }, []);

    const loadConfig = async () => {
        try {
            const config = await invoke<AppConfig>('get_config');
            setConfig(config);
        } catch (error) {
            console.error('Failed to load config:', error);
            // Set a default config if loading fails
            setConfig({
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
            });
        }
    };

    const saveConfig = async () => {
        if (!config) return;

        setSaving(true);
        try {
            // Save config first
            await invoke('save_config', { newConfig: config });

            // Then update autostart
            try {
                await invoke('set_autostart', { enabled: config.autostart });
            } catch (error) {
                console.error('Failed to update autostart:', error);
            }

            // Then register hotkey
            try {
                // Unregister all existing hotkeys
                await unregisterAll();

                // Register new hotkey if not empty
                if (config.hotkey) {
                    await register(config.hotkey, () => {
                        console.log('Hotkey pressed');
                        invoke('show_main_window');
                    });
                }
            } catch (error) {
                console.error('Failed to register hotkey:', error);
            }

            // Finally restart the app
            await invoke('restart_app');
        } catch (error) {
            console.error('Failed to save config:', error);
            // Don't restart if config save failed
        } finally {
            setSaving(false);
        }
    };

    const handleClose = async () => {
        await invoke('close_settings_window');
    };

    const updateConfig = (updates: Partial<MouseMonitorConfig>) => {
        if (!config) return;

        setConfig({
            ...config,
            mouse_monitor: {
                ...config.mouse_monitor,
                ...updates,
            },
        });
    };

    const toggleAutostart = () => {
        if (!config) return;

        setConfig({
            ...config,
            autostart: !config.autostart,
        });
    };

    const toggleAnalytics = async () => {
        if (!config) return;

        const newAnalyticsState = !config.analytics_enabled;

        try {
            if (newAnalyticsState) {
                await invoke('accept_analytics_consent');
            } else {
                await invoke('decline_analytics_consent');
            }

            // Update local state
            setConfig({
                ...config,
                analytics_enabled: newAnalyticsState,
            });
        } catch (error) {
            console.error('Failed to update analytics consent:', error);
        }
    };

    const startKeyListener = () => {
        setIsListening(true);
        setCurrentHotkey('Press keys...');
        window.addEventListener('keydown', handleKeyDown, true);
        window.addEventListener('keyup', handleKeyUp, true);
    };

    const stopKeyListener = () => {
        setIsListening(false);
        setCurrentHotkey('');
        window.removeEventListener('keydown', handleKeyDown, true);
        window.removeEventListener('keyup', handleKeyUp, true);
    };

    const clearHotkey = () => {
        if (!config) return;
        setConfig({
            ...config,
            hotkey: '',
        });
    };

    const addWhitelistItem = () => {
        if (!config || !newWhitelistItem.trim()) return;

        const currentWhitelist = config.mouse_monitor.whitelist || [];
        if (!currentWhitelist.includes(newWhitelistItem.trim())) {
            updateConfig({
                whitelist: [...currentWhitelist, newWhitelistItem.trim()]
            });
        }
        setNewWhitelistItem('');
    };

    const removeWhitelistItem = (itemToRemove: string) => {
        if (!config) return;

        const currentWhitelist = config.mouse_monitor.whitelist || [];
        updateConfig({
            whitelist: currentWhitelist.filter(item => item !== itemToRemove)
        });
    };

    const buildHotkeyString = (e: KeyboardEvent): string => {
        const parts: string[] = [];

        // Add modifier keys in the correct order
        if (e.ctrlKey) parts.push('Ctrl');
        if (e.altKey) parts.push('Alt');
        if (e.shiftKey) parts.push('Shift');
        if (e.metaKey) parts.push('Meta');

        // Get friendly key name for the current key
        let keyName = '';
        const code = e.code;

        // Handle special cases
        if (code.startsWith('Key')) {
            keyName = code.replace('Key', '');
        } else if (code.startsWith('Digit')) {
            keyName = code.replace('Digit', '');
        } else if (code === 'Space') {
            keyName = 'Space';
        } else if (code.startsWith('Arrow')) {
            keyName = code.replace('Arrow', ''); // Up, Down, Left, Right
        } else if (code === 'Escape') {
            keyName = 'Esc';
        } else if (code === 'Backspace') {
            keyName = 'Backspace';
        } else if (code === 'Tab') {
            keyName = 'Tab';
        } else if (code === 'Enter') {
            keyName = 'Enter';
        } else if (code === 'ControlLeft' || code === 'ControlRight') {
            return parts.join('+'); // Only return modifiers for Control key
        } else if (code === 'AltLeft' || code === 'AltRight') {
            return parts.join('+'); // Only return modifiers for Alt key
        } else if (code === 'ShiftLeft' || code === 'ShiftRight') {
            return parts.join('+'); // Only return modifiers for Shift key
        } else if (code === 'MetaLeft' || code === 'MetaRight') {
            return parts.join('+'); // Only return modifiers for Meta key
        } else {
            keyName = code;
        }

        // Add the key name if it's not a modifier key
        if (keyName) {
            parts.push(keyName);
        }

        return parts.join('+');
    };

    const handleKeyDown = (e: KeyboardEvent) => {
        // Always prevent default behavior to stop special character input
        e.preventDefault();
        e.stopPropagation();

        // Update the current hotkey display
        const hotkeyString = buildHotkeyString(e);
        setCurrentHotkey(hotkeyString || 'Press keys...');

        // Skip if we're only detecting a modifier key press
        if (['ControlLeft', 'ControlRight', 'ShiftLeft', 'ShiftRight',
            'AltLeft', 'AltRight', 'MetaLeft', 'MetaRight'].includes(e.code)) {
            return;
        }

        // If it's a non-modifier key, finalize the hotkey
        if (!['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
            // Update config with the new shortcut
            setConfig(prev => prev ? { ...prev, hotkey: hotkeyString } : null);

            // Stop listening for keys
            stopKeyListener();
        }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
        // Update current hotkey display on key up as well
        // (especially important for showing modifier state changes)
        const hotkeyString = buildHotkeyString(e);
        setCurrentHotkey(hotkeyString || 'Press keys...');
    };

    if (!config) {
        return (
            <div className="flex h-full items-center justify-center bg-background text-foreground">
                <div className="flex flex-col items-center space-y-4">
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
                    <p className="text-muted-foreground">Loading settings...</p>
                </div>
            </div>
        );
    }

    return (
        <div className="flex flex-col h-full bg-background text-foreground select-none">
            <style>{`
                /* Remove number input spinners */
                input[type=number]::-webkit-inner-spin-button, 
                input[type=number]::-webkit-outer-spin-button {
                    -webkit-appearance: none;
                    margin: 0;
                }
                input[type=number] {
                    -moz-appearance: textfield;
                }

                /* Custom scrollbar styling */
                ::-webkit-scrollbar {
                    width: 8px;
                    height: 8px;
                }
                
                ::-webkit-scrollbar-track {
                    background: transparent;
                }
                
                ::-webkit-scrollbar-thumb {
                    background: hsl(var(--border));
                    border-radius: 4px;
                    transition: background 0.2s;
                }
                
                ::-webkit-scrollbar-thumb:hover {
                    background: hsl(var(--primary) / 0.5);
                }
                
                /* Firefox scrollbar styling */
                * {
                    scrollbar-width: thin;
                    scrollbar-color: hsl(var(--border)) transparent;
                }
            `}</style>

            {/* Title Bar */}
            <div className="flex justify-between items-center p-4 border-b border-border/40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 sticky top-0 z-50 min-h-[60px]" data-tauri-drag-region>
                <div className="flex items-center gap-2" data-tauri-drag-region>
                    <div className="bg-primary/10 p-2 rounded-lg" data-tauri-drag-region>
                        <Settings className="h-5 w-5 text-primary" />
                    </div>
                    <div>
                        <h1 className="text-lg font-semibold tracking-tight" data-tauri-drag-region>Settings</h1>
                        <p className="text-xs text-muted-foreground" data-tauri-drag-region>Manage your application preferences</p>
                    </div>
                </div>
                <Button
                    variant="ghost"
                    size="icon"
                    onClick={handleClose}
                    className="h-8 w-8 hover:bg-destructive/10 hover:text-destructive rounded-full transition-colors"
                >
                    <X className="h-4 w-4" />
                </Button>
            </div>

            {/* Settings Content */}
            <div className="flex-grow p-6 overflow-auto space-y-6 scrollbar-thin scrollbar-thumb-border scrollbar-track-transparent">

                {/* General Settings */}
                <div className="grid gap-6">
                    <Card>
                        <CardHeader>
                            <CardTitle className="text-base flex items-center gap-2">
                                <Monitor className="w-4 h-4 text-primary" />
                                General Settings
                            </CardTitle>
                            <CardDescription>Configure startup and privacy behavior</CardDescription>
                        </CardHeader>
                        <CardContent className="grid gap-6">
                            <div className="flex items-center justify-between space-x-2">
                                <Label htmlFor="startup" className="flex flex-col space-y-1">
                                    <span>Start on System Startup</span>
                                    <span className="font-normal text-xs text-muted-foreground">Automatically launch the app when you sign in</span>
                                </Label>
                                <Switch
                                    id="startup"
                                    checked={config.autostart}
                                    onCheckedChange={toggleAutostart}
                                />
                            </div>

                            <div className="flex items-center justify-between space-x-2">
                                <Label htmlFor="analytics" className="flex flex-col space-y-1">
                                    <span>Enable Analytics</span>
                                    <span className="font-normal text-xs text-muted-foreground">Help improve Holdem by sharing anonymous usage data</span>
                                </Label>
                                <Switch
                                    id="analytics"
                                    checked={config.analytics_enabled}
                                    onCheckedChange={toggleAnalytics}
                                />
                            </div>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader>
                            <CardTitle className="text-base flex items-center gap-2">
                                <Keyboard className="w-4 h-4 text-primary" />
                                Shortcuts
                            </CardTitle>
                            <CardDescription>Customize how you interact with the app</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <div className="space-y-2">
                                <Label>Show Window Hotkey</Label>
                                <div className="flex gap-2">
                                    <div className={`
                                        flex-1 h-10 px-3 rounded-md border flex items-center justify-center font-mono text-sm shadow-sm transition-colors
                                        ${isListening
                                            ? 'border-primary ring-1 ring-primary bg-primary/5 text-primary'
                                            : 'border-input bg-background text-foreground'
                                        }
                                    `}>
                                        {isListening ? currentHotkey : (config.hotkey || <span className="text-muted-foreground italic">None set</span>)}
                                    </div>
                                    <Button
                                        onClick={isListening ? stopKeyListener : startKeyListener}
                                        variant={isListening ? "destructive" : "default"}
                                        className="w-24 shrink-0 shadow-sm"
                                    >
                                        {isListening ? 'Stop' : 'Set'}
                                    </Button>
                                    <Button
                                        onClick={clearHotkey}
                                        variant="outline"
                                        className="w-20 shrink-0 shadow-sm"
                                        disabled={!config.hotkey}
                                    >
                                        Clear
                                    </Button>
                                </div>
                                <p className="text-xs text-muted-foreground flex items-center gap-1.5">
                                    <Info className="w-3 h-3" />
                                    {isListening
                                        ? 'Press desired key combination... Press Stop when done'
                                        : 'Global shortcut to bring the app to foreground'}
                                </p>
                            </div>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader>
                            <CardTitle className="text-base flex items-center gap-2">
                                <Monitor className="w-4 h-4 text-primary" />
                                Mouse Monitor
                            </CardTitle>
                            <CardDescription>Fine-tune shake detection sensitivity</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-6">
                            <div className="grid grid-cols-2 gap-4">
                                <div className="space-y-2">
                                    <Label className="text-xs font-medium text-muted-foreground uppercase tracking-wider">Sensitivity</Label>
                                    <div className="space-y-1">
                                        <Label htmlFor="shakes">Required Shakes</Label>
                                        <Input
                                            id="shakes"
                                            type="number"
                                            value={config.mouse_monitor.required_shakes}
                                            onChange={(e) => updateConfig({ required_shakes: parseInt(e.target.value) })}
                                            className="font-mono"
                                        />
                                    </div>
                                    <div className="space-y-1">
                                        <Label htmlFor="threshold">Shake Threshold</Label>
                                        <Input
                                            id="threshold"
                                            type="number"
                                            value={config.mouse_monitor.shake_threshold}
                                            onChange={(e) => updateConfig({ shake_threshold: parseInt(e.target.value) })}
                                            className="font-mono"
                                        />
                                    </div>
                                </div>
                                <div className="space-y-2">
                                    <Label className="text-xs font-medium text-muted-foreground uppercase tracking-wider">Timing</Label>
                                    <div className="space-y-1">
                                        <Label htmlFor="limit">Time Limit (ms)</Label>
                                        <Input
                                            id="limit"
                                            type="number"
                                            value={config.mouse_monitor.shake_time_limit}
                                            onChange={(e) => updateConfig({ shake_time_limit: parseInt(e.target.value) })}
                                            className="font-mono"
                                        />
                                    </div>
                                    <div className="space-y-1">
                                        <Label htmlFor="delay">Close Delay (ms)</Label>
                                        <Input
                                            id="delay"
                                            type="number"
                                            value={config.mouse_monitor.window_close_delay}
                                            onChange={(e) => updateConfig({ window_close_delay: parseInt(e.target.value) })}
                                            className="font-mono"
                                        />
                                    </div>
                                </div>
                            </div>

                            <div className="space-y-3 pt-4 border-t">
                                <div className="space-y-1">
                                    <Label>Whitelisted Apps</Label>
                                    <p className="text-xs text-muted-foreground">
                                        Shake detection will only be active when one of these apps is focused.
                                        Accepts process names (e.g. <code className="bg-muted px-1 py-0.5 rounded text-foreground">brave</code>, <code className="bg-muted px-1 py-0.5 rounded text-foreground">brave.exe</code>). Case-insensitive.
                                    </p>
                                </div>

                                <div className="flex gap-2">
                                    <Input
                                        type="text"
                                        value={newWhitelistItem}
                                        onChange={(e) => setNewWhitelistItem(e.target.value)}
                                        placeholder="Add process name..."
                                        onKeyDown={(e) => {
                                            if (e.key === 'Enter') {
                                                addWhitelistItem();
                                            }
                                        }}
                                        className="font-mono text-sm"
                                    />
                                    <Button
                                        onClick={addWhitelistItem}
                                        variant="secondary"
                                        disabled={!newWhitelistItem.trim()}
                                        className="shrink-0"
                                    >
                                        <Plus className="h-4 w-4 mr-2" />
                                        Add
                                    </Button>
                                </div>

                                <div className="bg-muted/30 rounded-lg border min-h-[100px] max-h-[200px] overflow-y-auto p-1">
                                    {(config.mouse_monitor.whitelist || []).length > 0 ? (
                                        <div className="space-y-1">
                                            {config.mouse_monitor.whitelist.map((app, index) => (
                                                <div
                                                    key={index}
                                                    className="group flex items-center justify-between p-2 rounded-md hover:bg-background hover:shadow-sm hover:border-border/50 border border-transparent transition-all"
                                                >
                                                    <div className="flex items-center gap-2 overflow-hidden">
                                                        <div className="h-6 w-6 rounded bg-primary/10 flex items-center justify-center shrink-0">
                                                            <span className="text-xs font-mono font-bold text-primary">{app.charAt(0).toUpperCase()}</span>
                                                        </div>
                                                        <span className="text-sm font-mono truncate">{app}</span>
                                                    </div>
                                                    <Button
                                                        variant="ghost"
                                                        size="sm"
                                                        onClick={() => removeWhitelistItem(app)}
                                                        className="h-7 w-7 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-all rounded-md"
                                                    >
                                                        <Trash2 className="h-4 w-4" />
                                                    </Button>
                                                </div>
                                            ))}
                                        </div>
                                    ) : (
                                        <div className="flex flex-col items-center justify-center py-8 text-center px-4">
                                            <div className="h-8 w-8 rounded-full bg-muted flex items-center justify-center mb-2">
                                                <Monitor className="h-4 w-4 text-muted-foreground" />
                                            </div>
                                            <p className="text-sm font-medium text-foreground">No apps whitelisted</p>
                                            <p className="text-xs text-muted-foreground mt-1">Cursor shaking will be ignored globally</p>
                                        </div>
                                    )}
                                </div>
                            </div>
                        </CardContent>
                    </Card>
                </div>
            </div>

            {/* Footer */}
            <div className="p-4 border-t border-border bg-background/95 backdrop-blur z-50">
                <Button
                    onClick={saveConfig}
                    disabled={saving}
                    className="w-full sm:w-auto ml-auto px-8 shadow-sm flex items-center gap-2"
                >
                    {saving ? (
                        <>
                            <div className="h-4 w-4 animate-spin rounded-full border-2 border-primary-foreground border-t-transparent"></div>
                            <span>Saving...</span>
                        </>
                    ) : (
                        'Save Settings'
                    )}
                </Button>
            </div>
        </div>
    );
}

// Simple Switch component helper since it was missing from imports
function Switch({ id, checked, onCheckedChange }: { id: string, checked: boolean, onCheckedChange: () => void }) {
    return (
        <button
            id={id}
            onClick={onCheckedChange}
            className={`
                relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent 
                transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 
                focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50
                ${checked ? 'bg-primary' : 'bg-input'}
            `}
        >
            <span
                className={`
                    pointer-events-none block h-5 w-5 rounded-full bg-background shadow-lg ring-0 transition-transform
                    ${checked ? 'translate-x-5' : 'translate-x-0'}
                `}
            />
        </button>
    );
}
