<script lang="ts">
  import { onMount } from 'svelte';
  import { protectionSet } from '@/types/protection';
  import { configStore } from '@/stores/config.svelte';

  let enabled = $state(false);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let success = $state<string | null>(null);

  // Sync with store
  $effect(() => {
    if (configStore.config?.ui) {
      enabled = configStore.config.ui.protection ?? false;
    }
  });

  async function toggleProtection() {
    try {
      loading = true;
      error = null;
      success = null;
      
      const newState = !enabled;
      await protectionSet(newState);
      enabled = newState;
      
      success = newState ? 'Защита включена' : 'Защита выключена';
      setTimeout(() => success = null, 3000);
      
      // Update store
      if (configStore.config) {
        configStore.config = {
          ...configStore.config,
          ui: { ...configStore.config.ui, protection: newState },
        };
      }
    } catch (e) {
      error = `Ошибка: ${e}`;
      console.error('Protection toggle error:', e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    if (!configStore.config && !configStore.loading) {
      configStore.load();
    }
  });
</script>

<div class="protection-settings p-6 max-w-2xl">
  <h2 class="text-2xl font-semibold mb-2">Защита</h2>
  <p class="text-gray-600 mb-6">
    Маскировка окна от демонстрации и записи экрана.
  </p>

  {#if configStore.loading}
    <div class="flex items-center gap-2 text-blue-600">
      <div class="w-4 h-4 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
      <span>Загрузка...</span>
    </div>
  {:else}
    <div class="space-y-6">
      <div class="p-6 bg-gray-50 border border-gray-200 rounded-lg">
        <div class="flex items-start justify-between gap-4">
          <div class="flex-1">
            <h3 class="font-semibold text-lg mb-2">Режим защиты</h3>
            <p class="text-gray-600 text-sm mb-4">
              Окно оверлея не видно в демонстрации экрана, записи видео и скриншотах (Windows).
              Используйте это для конфиденциальных встреч.
            </p>
            <div class="flex items-center gap-3">
              <button
                onclick={toggleProtection}
                disabled={loading}
                class="relative inline-flex h-8 w-16 items-center rounded-full transition-colors disabled:opacity-50 {enabled ? 'bg-green-600' : 'bg-gray-300'}"
                aria-label="Toggle protection mode"
              >
                <span class="inline-block h-6 w-6 transform rounded-full bg-white transition-transform {enabled ? 'translate-x-9' : 'translate-x-1'}"></span>
              </button>
              <span class="font-medium {enabled ? 'text-green-700' : 'text-gray-500'}">
                {enabled ? 'Включено' : 'Выключено'}
              </span>
            </div>
          </div>
          
          {#if enabled}
            <div class="flex-shrink-0 w-12 h-12 flex items-center justify-center rounded-full bg-green-100 text-green-600">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
              </svg>
            </div>
          {/if}
        </div>
      </div>

      {#if error || configStore.error}
        <div class="p-3 bg-red-50 border border-red-200 rounded-lg text-red-800 text-sm">
          {error || configStore.error}
        </div>
      {/if}

      {#if success}
        <div class="p-3 bg-green-50 border border-green-200 rounded-lg text-green-800 text-sm">
          {success}
        </div>
      {/if}

      <div class="p-4 bg-blue-50 border border-blue-200 rounded-lg">
        <div class="flex gap-3">
          <div class="flex-shrink-0">
            <svg class="w-5 h-5 text-blue-600" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
            </svg>
          </div>
          <div class="text-sm text-blue-800">
            <strong>Примечание:</strong> Защита работает только в Windows. Окно будет скрыто от системных инструментов захвата экрана, но может быть видно в некоторых сторонних приложениях.
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
