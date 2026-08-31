<script lang="ts">
  import { onMount } from 'svelte';
  import { ttsSetMode } from '@/types/commands';
  import { configStore } from '@/stores/config.svelte';

  type TtsMode = 'off' | 'auto' | 'hotkey';

  let mode = $state<TtsMode>('off');
  let loading = $state(false);
  let error = $state<string | null>(null);

  const modes: { value: TtsMode; label: string }[] = [
    { value: 'off', label: 'Выкл' },
    { value: 'auto', label: 'Авто (стриминг)' },
    { value: 'hotkey', label: 'По хоткею (Ctrl+T)' },
  ];

  // Reactive effect: sync local state with store
  $effect(() => {
    if (configStore.config?.tts?.mode) {
      mode = configStore.config.tts.mode;
    }
  });

  async function saveMode(newMode: TtsMode) {
    try {
      loading = true;
      error = null;
      await ttsSetMode(newMode);
      mode = newMode;
      configStore.updateTts({ mode: newMode });
    } catch (e) {
      error = `Ошибка сохранения: ${e}`;
      console.error('TTS save error:', e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    // Load from store if not already loaded
    if (!configStore.config && !configStore.loading) {
      configStore.load();
    }
  });
</script>

<div class="tts-controls p-6 max-w-2xl">
  <h2 class="text-2xl font-semibold mb-2">Озвучка ответов</h2>
  <p class="text-gray-600 mb-6">
    Синтез речи ответов ИИ (Cartesia). Авто — озвучка включается сразу; по хоткею — только при нажатии Ctrl+T.
  </p>

  {#if configStore.loading}
    <div class="flex items-center gap-2 text-blue-600">
      <div class="w-4 h-4 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
      <span>Загрузка...</span>
    </div>
  {:else}
    <div class="space-y-4">
      <div class="flex flex-col gap-2">
        <label for="tts-mode" class="font-medium">Режим озвучки</label>
        <select
          id="tts-mode"
          bind:value={mode}
          onchange={() => saveMode(mode)}
          class="px-4 py-2 border border-gray-300 rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
          disabled={loading}
        >
          {#each modes as m}
            <option value={m.value}>{m.label}</option>
          {/each}
        </select>
      </div>

      {#if error || configStore.error}
        <div class="p-3 bg-red-50 border border-red-200 rounded-lg text-red-800 text-sm">
          {error || configStore.error}
        </div>
      {/if}

      <p class="text-sm text-gray-500 mt-4">
        Ключ API задаётся в config.toml ([tts] api_key)
      </p>
    </div>
  {/if}
</div>
