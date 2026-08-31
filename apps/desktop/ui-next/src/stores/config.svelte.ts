// Global config store using Svelte 5 runes

import { getConfig, type AppConfig } from '@/types/commands';

// Create a reactive state object
class ConfigStore {
  config = $state<AppConfig | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);

  async load() {
    try {
      this.loading = true;
      this.error = null;
      this.config = await getConfig();
    } catch (e) {
      this.error = `Failed to load config: ${e}`;
      console.error('Config store load error:', e);
    } finally {
      this.loading = false;
    }
  }

  // Partial update helpers
  updateTts(tts: Partial<AppConfig['tts']>) {
    if (this.config) {
      this.config = {
        ...this.config,
        tts: { ...this.config.tts, ...tts },
      };
    }
  }

  updateAudio(audio: Partial<AppConfig['audio']>) {
    if (this.config) {
      this.config = {
        ...this.config,
        audio: { ...this.config.audio, ...audio },
      };
    }
  }

  reset() {
    this.config = null;
    this.error = null;
  }
}

// Export singleton instance
export const configStore = new ConfigStore();
