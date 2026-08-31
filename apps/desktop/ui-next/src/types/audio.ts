// Extended Tauri command types for Audio Settings

import { invoke as tauriInvoke } from '@tauri-apps/api/core';

// Audio Settings Types
export type AudioSource = 'system+mic' | 'system' | 'mic';
export type AudioMode = 'vad' | 'manual';

export interface AudioConfig {
  source?: AudioSource;
  mode?: AudioMode;
  micDevice?: string | null;
}

// Audio Commands
export async function listAudioDevices(): Promise<string[]> {
  return tauriInvoke<string[]>('list_audio_devices');
}

export async function updateAudioSettings(
  source: AudioSource,
  mode: AudioMode,
  micDevice: string | null
): Promise<void> {
  return tauriInvoke('update_audio_settings', { source, mode, micDevice });
}
