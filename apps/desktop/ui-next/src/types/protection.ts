// Protection settings types

import { invoke as tauriInvoke } from '@tauri-apps/api/core';

export async function protectionSet(on: boolean): Promise<void> {
  return tauriInvoke('protection_set', { on });
}
