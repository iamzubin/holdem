import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface ShakeConfig {
  shake_threshold: number;
  shake_time_limit: number;
  required_shakes: number;
}

export function useShakeThreshold() {
  const [config, setConfig] = useState<ShakeConfig | null>(null);

  useEffect(() => {
    invoke<any>('get_config').then((res) => {
      setConfig(res.mouse_monitor || {
        shake_threshold: 50,
        shake_time_limit: 400,
        required_shakes: 3
      });
    }).catch(console.error);
  }, []);

  const updateShakeConfig = useCallback(async (updates: Partial<ShakeConfig>) => {
    try {
      const fullConfig = await invoke<any>('get_config');
      const newConfig = {
        ...fullConfig,
        mouse_monitor: {
          ...fullConfig.mouse_monitor,
          ...updates
        }
      };
      await invoke('save_config', { config: newConfig });
      setConfig(newConfig.mouse_monitor);
    } catch (err) {
      console.error('Failed to update shake config', err);
    }
  }, []);

  return { shakeConfig: config, updateShakeConfig };
}
