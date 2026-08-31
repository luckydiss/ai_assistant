// Type-safe Tauri command bindings
// This file is manually maintained until Tauri Specta integration (Phase 2)

import { invoke as tauriInvoke } from '@tauri-apps/api/core';

// ============================================================================
// POC Commands
// ============================================================================

export async function greet(name: string): Promise<string> {
  return tauriInvoke<string>('greet', { name });
}

// ============================================================================
// Config Commands
// ============================================================================

export interface TtsConfig {
  mode?: 'off' | 'auto' | 'hotkey';
}

export interface AudioConfig {
  source?: 'system+mic' | 'system' | 'mic';
  mode?: 'vad' | 'manual';
  micDevice?: string | null;
}

export interface UiConfig {
  accent?: string;
  opacity?: number;
  protection?: boolean;
}

export interface WindowConfig {
  move_step?: number;
  resize_step?: number;
}

export interface ChatConfig {
  order?: 'bottom' | 'top';
  font_size?: number;
  code_theme?: 'github-dark' | 'monokai';
  code_scroll?: boolean;
  autoscroll?: boolean;
  autoscroll_speed?: number;
  collapse_transcripts?: boolean;
  collapse_last?: boolean;
  compact_quick?: boolean;
  cancel_on_resend?: boolean;
  cancel_mode?: 'drop' | 'keep';
}

export interface AppConfig {
  tts?: TtsConfig;
  audio?: AudioConfig;
  ui?: UiConfig;
  window?: WindowConfig;
  chat?: ChatConfig;
  // Add more config sections as needed
}

export async function getConfig(): Promise<AppConfig> {
  return tauriInvoke<AppConfig>('get_config');
}

export async function ttsSetMode(mode: 'off' | 'auto' | 'hotkey'): Promise<void> {
  return tauriInvoke('tts_set_mode', { mode });
}

// Generic config setter
export async function cfgSet(section: string, key: string, value: string | number | boolean): Promise<void> {
  return tauriInvoke('cfg_set', { section, key, value });
}

// ============================================================================
// Type Guards
// ============================================================================

export function isTtsMode(value: unknown): value is 'off' | 'auto' | 'hotkey' {
  return typeof value === 'string' && ['off', 'auto', 'hotkey'].includes(value);
}
