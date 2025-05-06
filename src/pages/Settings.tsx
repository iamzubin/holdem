import { invoke } from '@tauri-apps/api/core';
import { X } from 'lucide-react';
import { useEffect, useState } from 'react';
import { Updater } from '../components/Updater';

interface MouseMonitorConfig {
    required_shakes: number;
    shake_time_limit: number;
    shake_threshold: number;
    window_close_delay: number;
}

interface AppConfig {
    mouse_monitor: MouseMonitorConfig;
}

export default function SettingsPage() {
    const [config, setConfig] = useState<AppConfig | null>(null);
    const [saving, setSaving] = useState(false);

    useEffect(() => {
        loadConfig();
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
            await invoke('save_config', { newConfig: config });
            // Restart the app
            await invoke('restart_app');
        } catch (error) {
            console.error('Failed to save config:', error);
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

    if (!config) {
        return <div>Loading...</div>;
    }

    return (
        <div className="flex flex-col h-full bg-background text-foreground">
            {/* Title Bar */}
            <div className="flex justify-between items-center p-2 border-b border-border" data-tauri-drag-region>
                <h1 className="text-lg font-semibold" data-tauri-drag-region>Settings</h1>
                <button 
                    onClick={handleClose}
                    className="text-foreground hover:bg-red-500 hover:text-background rounded h-5 w-5 flex items-center justify-center"
                >
                    <X className="h-4 w-4" />
                </button>
            </div>

            {/* Settings Content */}
            <div className="flex-grow p-4 overflow-auto">
                <div className="space-y-6">
                    <div className="bg-card p-4 rounded-lg shadow">
                        <h2 className="text-xl font-semibold mb-4">Mouse Monitor Settings</h2>
                        
                        <div className="space-y-4">
                            <div>
                                <label className="block text-sm font-medium text-foreground">
                                    Required Shakes
                                </label>
                                <input
                                    type="number"
                                    value={config.mouse_monitor.required_shakes}
                                    onChange={(e) => updateConfig({ required_shakes: parseInt(e.target.value) })}
                                    className="mt-1 block w-full rounded-md border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                                />
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-foreground">
                                    Shake Time Limit (ms)
                                </label>
                                <input
                                    type="number"
                                    value={config.mouse_monitor.shake_time_limit}
                                    onChange={(e) => updateConfig({ shake_time_limit: parseInt(e.target.value) })}
                                    className="mt-1 block w-full rounded-md border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                                />
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-foreground">
                                    Shake Threshold
                                </label>
                                <input
                                    type="number"
                                    value={config.mouse_monitor.shake_threshold}
                                    onChange={(e) => updateConfig({ shake_threshold: parseInt(e.target.value) })}
                                    className="mt-1 block w-full rounded-md border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                                />
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-foreground">
                                    Window Close Delay (ms)
                                </label>
                                <input
                                    type="number"
                                    value={config.mouse_monitor.window_close_delay}
                                    onChange={(e) => updateConfig({ window_close_delay: parseInt(e.target.value) })}
                                    className="mt-1 block w-full rounded-md border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {/* Footer */}
            <div className="flex justify-end p-4 border-t border-border">
                <Updater />
                <button
                    onClick={saveConfig}
                    disabled={saving}
                    className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50"
                >
                    {saving ? 'Saving...' : 'Save Settings'}
                </button>
            </div>
        </div>
    );
} 