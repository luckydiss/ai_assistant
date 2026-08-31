<script lang="ts">
  import { onMount } from 'svelte';
  import { listAudioDevices, updateAudioSettings, type AudioSource, type AudioMode } from '@/types/audio';
  import { configStore } from '@/stores/config.svelte';

  let source = $state<AudioSource>('system+mic');
  let mode = $state<AudioMode>('manual');
  let micDevice = $state<string>('');
  let devices = $state<string[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let success = $state(false);

  const sourceOptions: { value: AudioSource; label: string }[] = [
    { value: 'system+mic', label: 'Система + микрофон' },
    { value: 'system', label: 'Только система' },
    { value: 'mic', label: 'Только микрофон' },
  ];

  const modeOptions: { value: AudioMode; label: string }[] = [
    { value: 'vad', label: 'Авто (VAD)' },
    { value: 'manual', label: 'Ручной' },
  ];

  // Reactive effect: sync local state with store
  $effect(() => {
    if (configStore.config?.audio) {
      source = configStore.config.audio.source || 'system+mic';
      mode = configStore.config.audio.mode || 'manual';
      micDevice = configStore.config.audio.micDevice || '';
    }
  });

  async function loadDevices() {
    try {
      devices = await listAudioDevices();
    } catch (e) {
      console.error('Audio devices load error:', e);
    }
  }

  async function save() {
    try {
      loading = true;
      error = null;
      success = false;
      
      await updateAudioSettings(source, mode, micDevice || null);
      
      // Update store
      configStore.updateAudio({ source, mode, micDevice: micDevice || null });
      
      success = true;
      setTimeout(() => success = false, 3000);
    } catch (e) {
      error = `Ошибка сохранения: ${e}`;
      console.error('Audio settings save error:', e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    // Load config from store if not already loaded
    if (!configStore.config && !configStore.loading) {
      configStore.load();
    }
    loadDevices();
  });
</script>

<div class="audio-settings p-6 max-w-2xl">
  <h2 class="text-2xl font-semibold mb-2">Запись звука</h2>
  <p class="text-gray-600 mb-6">
    Источник аудио и режим распознавания речи.
  </p>

  {#if configStore.loading && devices.length === 0}
    <div class="flex items-center gap-2 text-blue-600">
      <div class="w-4 h-4 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
      <span>Загрузка...</span>
    </div>
  {:else}
    <div class="space-y-6">
      <div class="flex flex-col gap-2">
        <label for="audio-source" class="font-medium">Каналы записи</label>
        <p class="text-sm text-gray-600 mb-2">Откуда брать звук для расшифровки</p>
        <select
          id="audio-source"
          bind:value={source}
          class="px-4 py-2 border border-gray-300 rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
          disabled={loading}
        >
          {#each sourceOptions as opt}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </div>

      <div class="flex flex-col gap-2">
        <label for="audio-mode" class="font-medium">Режим записи</label>
        <select
          id="audio-mode"
          bind:value={mode}
          class="px-4 py-2 border border-gray-300 rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
          disabled={loading}
        >
          {#each modeOptions as opt}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </div>

      <div class="flex flex-col gap-2">
        <label for="mic-device" class="font-medium">Микрофон</label>
        <select
          id="mic-device"
          bind:value={micDevice}
          class="px-4 py-2 border border-gray-300 rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
          disabled={loading}
        >
          <option value="">по умолчанию</option>
          {#each devices as device}
            <option value={device}>{device}</option>
          {/each}
        </select>
      </div>

      {#if error || configStore.error}
        <div class="p-3 bg-red-50 border border-red-200 rounded-lg text-red-800 text-sm">
          {error || configStore.error}
        </div>
      {/if}

      {#if success}
        <div class="p-3 bg-green-50 border border-green-200 rounded-lg text-green-800 text-sm">
          Настройки записи сохранены
        </div>
      {/if}

      <button
        onclick={save}
        disabled={loading}
        class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {loading ? 'Сохранение...' : 'Сохранить'}
      </button>
    </div>
  {/if}
</div>
