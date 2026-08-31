<script lang="ts">
  import { onMount } from 'svelte';
  import { hotkeysGet, setHotkey, type HotkeyAction } from '@/types/hotkeys';

  type HotkeyItem = {
    action: HotkeyAction;
    label: string;
    value: string;
    status: 'idle' | 'success' | 'error';
  };

  const ACTIONS: { action: HotkeyAction; label: string }[] = [
    { action: 'manual', label: 'Что сказать' },
    { action: 'hide', label: 'Скрыть оверлей' },
    { action: 'click_through', label: 'Click-through' },
    { action: 'mute', label: 'Mute' },
    { action: 'record', label: 'Запись' },
    { action: 'screenshot_full', label: 'Скриншот (весь)' },
    { action: 'screenshot_region', label: 'Скриншот (регион)' },
  ];

  let hotkeys = $state<HotkeyItem[]>(
    ACTIONS.map(a => ({ ...a, value: '', status: 'idle' as const }))
  );
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function loadHotkeys() {
    try {
      loading = true;
      error = null;
      const hk = await hotkeysGet();
      
      hotkeys = ACTIONS.map(a => ({
        ...a,
        value: hk[a.action] || '',
        status: 'idle' as const,
      }));
    } catch (e) {
      error = `Ошибка загрузки: ${e}`;
      console.error('Hotkeys load error:', e);
    } finally {
      loading = false;
    }
  }

  async function updateHotkey(index: number) {
    const item = hotkeys[index];
    try {
      item.status = 'idle';
      await setHotkey(item.action, item.value.trim());
      item.status = 'success';
      setTimeout(() => { item.status = 'idle'; }, 1500);
    } catch (e) {
      item.status = 'error';
      console.error('Hotkey update error:', e);
      setTimeout(() => { item.status = 'idle'; }, 3000);
    }
  }

  function handleKeyDown(e: KeyboardEvent, index: number) {
    if (e.key === 'Enter') {
      (e.target as HTMLInputElement).blur();
    }
  }

  onMount(() => {
    loadHotkeys();
  });
</script>

<div class="hotkeys-settings p-6 max-w-2xl">
  <h2 class="text-2xl font-semibold mb-2">Горячие клавиши</h2>
  <p class="text-gray-600 mb-6">
    Глобальные сочетания клавиш. Изменения применяются автоматически.
  </p>

  {#if loading}
    <div class="flex items-center gap-2 text-blue-600">
      <div class="w-4 h-4 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
      <span>Загрузка...</span>
    </div>
  {:else if error}
    <div class="p-3 bg-red-50 border border-red-200 rounded-lg text-red-800 text-sm">
      {error}
    </div>
  {:else}
    <div class="space-y-3">
      {#each hotkeys as item, index (item.action)}
        <div class="flex items-center gap-3 p-3 bg-white border border-gray-200 rounded-lg">
          <span class="flex-1 font-medium">{item.label}</span>
          <div class="flex items-center gap-2">
            <input
              type="text"
              bind:value={item.value}
              onchange={() => updateHotkey(index)}
              onkeydown={(e) => handleKeyDown(e, index)}
              placeholder="(пусто = отключено)"
              class="px-3 py-1.5 border border-gray-300 rounded bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm w-48"
            />
            <span class="w-5 text-center">
              {#if item.status === 'success'}
                <span class="text-green-600">✓</span>
              {:else if item.status === 'error'}
                <span class="text-red-600">!</span>
              {/if}
            </span>
          </div>
        </div>
      {/each}
    </div>

    <p class="text-sm text-gray-500 mt-6">
      Примеры: <code class="px-1.5 py-0.5 bg-gray-100 rounded text-xs">Ctrl+Shift+A</code>, 
      <code class="px-1.5 py-0.5 bg-gray-100 rounded text-xs">Alt+Space</code>, 
      <code class="px-1.5 py-0.5 bg-gray-100 rounded text-xs">F9</code>
    </p>
  {/if}
</div>
