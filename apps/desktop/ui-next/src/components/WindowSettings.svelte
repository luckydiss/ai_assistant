<script lang="ts">
  import { onMount } from 'svelte';
  import { cfgSet } from '@/types/commands';
  import { configStore } from '@/stores/config.svelte';

  const ACCENT_PRESETS = ['#f97316', '#3b82f6', '#8b5cf6', '#10b981', '#ec4899'];

  let accent = $state('#f97316');
  let customAccent = $state('#f97316');
  let opacity = $state(100);
  let moveStep = $state(50);
  let resizeStep = $state(50);
  let loading = $state(false);
  let error = $state<string | null>(null);

  // Sync with store
  $effect(() => {
    if (configStore.config?.ui) {
      accent = configStore.config.ui.accent || '#f97316';
      customAccent = accent;
      opacity = configStore.config.ui.opacity || 100;
    }
    if (configStore.config?.window) {
      moveStep = configStore.config.window.move_step || 50;
      resizeStep = configStore.config.window.resize_step || 50;
    }
  });

  async function setAccent(color: string) {
    try {
      await cfgSet('ui', 'accent', color);
      accent = color;
      customAccent = color;
      // Apply immediately to document
      document.documentElement.style.setProperty('--accent', color);
    } catch (e) {
      error = `Ошибка: ${e}`;
      console.error('Accent set error:', e);
    }
  }

  async function setOpacity(value: number) {
    try {
      await cfgSet('ui', 'opacity', value);
      opacity = value;
    } catch (e) {
      error = `Ошибка: ${e}`;
      console.error('Opacity set error:', e);
    }
  }

  async function setMoveStep(value: number) {
    try {
      await cfgSet('window', 'move_step', value);
      moveStep = value;
    } catch (e) {
      error = `Ошибка: ${e}`;
      console.error('Move step set error:', e);
    }
  }

  async function setResizeStep(value: number) {
    try {
      await cfgSet('window', 'resize_step', value);
      resizeStep = value;
    } catch (e) {
      error = `Ошибка: ${e}`;
      console.error('Resize step set error:', e);
    }
  }

  onMount(() => {
    if (!configStore.config && !configStore.loading) {
      configStore.load();
    }
  });
</script>

<div class="window-settings p-6 max-w-2xl">
  <h2 class="text-2xl font-semibold mb-2">Окно</h2>
  <p class="text-gray-600 mb-6">
    Акцент, прозрачность и поведение окна оверлея.
  </p>

  {#if configStore.loading}
    <div class="flex items-center gap-2 text-blue-600">
      <div class="w-4 h-4 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
      <span>Загрузка...</span>
    </div>
  {:else}
    <div class="space-y-8">
      <!-- Accent Color -->
      <div>
        <div class="block font-medium mb-2">Акцентный цвет</div>
        <p class="text-sm text-gray-600 mb-4">
          Выберите базовый цвет для кнопок, выделений и активных состояний.
        </p>
        <div class="flex gap-3 items-center">
          {#each ACCENT_PRESETS as preset}
            <button
              onclick={() => setAccent(preset)}
              class="w-10 h-10 rounded-lg border-2 transition-all {accent === preset ? 'border-gray-800 scale-110' : 'border-gray-300 hover:scale-105'}"
              style="background-color: {preset}"
              title={preset}
              aria-label="Set accent color to {preset}"
            ></button>
          {/each}
          <div class="relative">
            <input
              type="color"
              bind:value={customAccent}
              onchange={() => setAccent(customAccent)}
              class="w-10 h-10 rounded-lg cursor-pointer border-2 border-gray-300"
              aria-label="Custom accent color"
            />
            <span class="absolute -bottom-5 left-0 text-xs text-gray-500">Свой</span>
          </div>
        </div>
      </div>

      <!-- Opacity -->
      <div>
        <label for="opacity" class="block font-medium mb-2">Непрозрачность интерфейса</label>
        <p class="text-sm text-gray-600 mb-4">
          Настройте прозрачность оверлея во время встречи.
        </p>
        <div class="flex items-center gap-4">
          <input
            id="opacity"
            type="range"
            min="10"
            max="100"
            step="5"
            bind:value={opacity}
            onchange={() => setOpacity(opacity)}
            class="flex-1"
          />
          <span class="w-16 text-right font-medium">{opacity}%</span>
        </div>
      </div>

      <!-- Move Step -->
      <div>
        <label for="move-step" class="block font-medium mb-2">Шаг перемещения окна</label>
        <p class="text-sm text-gray-600 mb-4">
          Количество пикселей, на которое окно перемещается при нажатии горячих клавиш.
        </p>
        <div class="flex items-center gap-4">
          <input
            id="move-step"
            type="range"
            min="10"
            max="200"
            step="10"
            bind:value={moveStep}
            onchange={() => setMoveStep(moveStep)}
            class="flex-1"
          />
          <span class="w-16 text-right font-medium">{moveStep} px</span>
        </div>
      </div>

      <!-- Resize Step -->
      <div>
        <label for="resize-step" class="block font-medium mb-2">Шаг изменения размера окна</label>
        <p class="text-sm text-gray-600 mb-4">
          Количество пикселей, на которое изменяется размер окна при нажатии горячих клавиш.
        </p>
        <div class="flex items-center gap-4">
          <input
            id="resize-step"
            type="range"
            min="10"
            max="200"
            step="10"
            bind:value={resizeStep}
            onchange={() => setResizeStep(resizeStep)}
            class="flex-1"
          />
          <span class="w-16 text-right font-medium">{resizeStep} px</span>
        </div>
      </div>

      {#if error || configStore.error}
        <div class="p-3 bg-red-50 border border-red-200 rounded-lg text-red-800 text-sm">
          {error || configStore.error}
        </div>
      {/if}
    </div>
  {/if}
</div>
