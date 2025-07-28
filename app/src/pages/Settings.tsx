import { invoke } from '@tauri-apps/api/core';
import { register, unregisterAll } from '@tauri-apps/plugin-global-shortcut';
import { X } from 'lucide-react';
import { useEffect, useState } from 'react';

interface MouseMonitorConfig {
    required_shakes: number;
    shake_time_limit: number;
    shake_threshold: number;
    window_close_delay: number;
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
        return <div>Loading...</div>;
    }

    return (
        <div className="flex flex-col h-full bg-background text-foreground text-xs">
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
            `}</style>
            {/* Title Bar */}
            <div className="flex justify-between items-center p-1 border-b border-border min-h-8" data-tauri-drag-region>
                <h1 className="text-base font-semibold" data-tauri-drag-region>Settings</h1>
                <button 
                    onClick={handleClose}
                    className="text-foreground hover:bg-red-500 hover:text-background rounded h-5 w-5 flex items-center justify-center"
                >
                    <X className="h-4 w-4" />
                </button>
            </div>

            {/* Settings Content */}
            <div className="flex-grow p-2 overflow-auto">
                <div className="space-y-3">
                    <div className="bg-card p-2 rounded shadow">
                        <h2 className="text-sm font-semibold mb-2">General Settings</h2>
                        <div className="space-y-2">
                            <div className="flex items-center justify-between">
                                <label className="text-xs font-medium text-foreground">
                                    Start on System Startup
                                </label>
                                <button
                                    onClick={toggleAutostart}
                                    className={`relative inline-flex h-4 w-8 items-center rounded-full transition-colors focus:outline-none focus:ring-1 focus:ring-primary focus:ring-offset-1 border`}
                                >
                                    <span
                                        className={`inline-block h-3 w-3 transform rounded-full bg-white transition-transform border ${config.autostart ? 'translate-x-4 border-primary' : 'translate-x-1 border-gray-400'}`}
                                    />
                                </button>
                            </div>
                            <div className="flex items-center justify-between">
                                <div>
                                    <label className="text-xs font-medium text-foreground">
                                        Enable Analytics
                                    </label>
                                    <p className="text-xs text-muted-foreground mt-0.5">
                                        Help improve Holdem by sharing anonymous usage data
                                    </p>
                                </div>
                                <button
                                    onClick={toggleAnalytics}
                                    className={`relative inline-flex h-4 w-8 items-center rounded-full transition-colors focus:outline-none focus:ring-1 focus:ring-primary focus:ring-offset-1 border`}
                                >
                                    <span
                                        className={`inline-block h-3 w-3 transform rounded-full bg-white transition-transform border ${config.analytics_enabled ? 'translate-x-4 border-primary' : 'translate-x-1 border-gray-400'}`}
                                    />
                                </button>
                            </div>
                            <div>
                                <label className="block text-xs font-medium text-foreground">
                                    Show Window Hotkey
                                </label>
                                <div className="flex gap-1 mt-0.5">
                                    <input
                                        type="text"
                                        value={isListening ? currentHotkey : config.hotkey}
                                        readOnly
                                        className={`block w-full rounded border-input bg-background px-2 py-1 text-xs ring-offset-background ${isListening ? 'border-primary animate-pulse' : ''}`}
                                    />
                                    <button
                                        onClick={isListening ? stopKeyListener : startKeyListener}
                                        className={`px-2 py-1 rounded ${isListening ? 'bg-red-500 text-white' : 'bg-primary text-primary-foreground'}`}
                                        style={{ minWidth: 70 }}
                                    >
                                        {isListening ? 'Stop' : 'Set Hotkey'}
                                    </button>
                                    <button
                                        onClick={clearHotkey}
                                        className="px-2 py-1 rounded bg-destructive text-destructive-foreground hover:bg-destructive/90"
                                        style={{ minWidth: 70 }}
                                    >
                                        Clear
                                    </button>
                                </div>
                                <p className="mt-0.5 text-xs text-muted-foreground">
                                    {isListening 
                                        ? 'Press key combination... Press Stop when done' 
                                        : 'Click "Set Hotkey" to start listening for key combinations'}
                                </p>
                            </div>
                        </div>
                    </div>
                    <div className="bg-card p-2 rounded shadow">
                        <h2 className="text-sm font-semibold mb-2">Mouse Monitor Settings</h2>
                        <div className="space-y-2">
                            <div>
                                <label className="block text-xs font-medium text-foreground">
                                    Required Shakes
                                </label>
                                <input
                                    type="number"
                                    value={config.mouse_monitor.required_shakes}
                                    onChange={(e) => updateConfig({ required_shakes: parseInt(e.target.value) })}
                                    className="mt-0.5 block w-full rounded border-input bg-background px-2 py-1 text-xs ring-offset-background focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                                />
                            </div>
                            <div>
                                <label className="block text-xs font-medium text-foreground">
                                    Shake Time Limit (ms)
                                </label>
                                <input
                                    type="number"
                                    value={config.mouse_monitor.shake_time_limit}
                                    onChange={(e) => updateConfig({ shake_time_limit: parseInt(e.target.value) })}
                                    className="mt-0.5 block w-full rounded border-input bg-background px-2 py-1 text-xs ring-offset-background focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                                />
                            </div>
                            <div>
                                <label className="block text-xs font-medium text-foreground">
                                    Shake Threshold
                                </label>
                                <input
                                    type="number"
                                    value={config.mouse_monitor.shake_threshold}
                                    onChange={(e) => updateConfig({ shake_threshold: parseInt(e.target.value) })}
                                    className="mt-0.5 block w-full rounded border-input bg-background px-2 py-1 text-xs ring-offset-background focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                                />
                            </div>
                            <div>
                                <label className="block text-xs font-medium text-foreground">
                                    Window Close Delay (ms)
                                </label>
                                <input
                                    type="number"
                                    value={config.mouse_monitor.window_close_delay}
                                    onChange={(e) => updateConfig({ window_close_delay: parseInt(e.target.value) })}
                                    className="mt-0.5 block w-full rounded border-input bg-background px-2 py-1 text-xs ring-offset-background focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            {/* Footer */}
            <div className="flex justify-end p-2 border-t border-border">
                <button
                    onClick={saveConfig}
                    disabled={saving}
                    className="px-3 py-1 bg-primary text-primary-foreground rounded hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:opacity-50 text-xs"
                >
                    {saving ? 'Saving...' : 'Save Settings'}
                </button>
            </div>
        </div>
    );
} 