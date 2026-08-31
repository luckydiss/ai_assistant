// Window/UI settings types

import { invoke as tauriInvoke } from '@tauri-apps/api/core';

export interface UiConfig {
  accent?: string;
  opacity?: number;
  protection?: boolean;
}

export interface WindowConfig {
  move_step?: number;
  resize_step?: number;
}

export async function setUiConfig(key: string, value: string | number | boolean): Promise<void> {
  return tauriInvoke('cfg_set', { section: 'ui', key, value });
}

export async function setWindowConfig(key: string, value: number): Promise<void> {
  return tauriInvoke('cfg_set', { section: 'window', key, value });
}
