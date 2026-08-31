// Hotkeys command types

import { invoke as tauriInvoke } from '@tauri-apps/api/core';

export type HotkeyAction = 
  | 'manual'
  | 'hide'
  | 'click_through'
  | 'mute'
  | 'record'
  | 'screenshot_full'
  | 'screenshot_region';

export type HotkeysMap = Record<HotkeyAction, string>;

export async function hotkeysGet(): Promise<HotkeysMap> {
  return tauriInvoke<HotkeysMap>('hotkeys_get');
}

export async function setHotkey(action: HotkeyAction, accel: string): Promise<void> {
  return tauriInvoke('set_hotkey', { action, accel });
}
