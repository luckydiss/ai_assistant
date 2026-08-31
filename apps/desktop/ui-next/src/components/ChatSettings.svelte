<script lang="ts">
  import { onMount } from 'svelte';
  import { cfgSet } from '@/types/commands';
  import { configStore } from '@/stores/config.svelte';

  type ChatOrder = 'bottom' | 'top';
  type CodeTheme = 'github-dark' | 'monokai';
  type CancelMode = 'drop' | 'keep';

  let order = $state<ChatOrder>('bottom');
  let fontSize = $state(14);
  let codeTheme = $state<CodeTheme>('github-dark');
  let codeScroll = $state(true);
  let autoscroll = $state(true);
  let autoscrollSpeed = $state(50);
  let collapseTranscripts = $state(false);
  let collapseLast = $state(false);
  let compactQuick = $state(false);
  let cancelOnResend = $state(true);
  let cancelMode = $state<CancelMode>('drop');
  let error = $state<string | null>(null);

  // Sync with store
  $effect(() => {
    if (configStore.config?.chat) {
      const c = configStore.config.chat;
      order = (c.order as ChatOrder) || 'bottom';
      fontSize = c.font_size || 14;
      codeTheme = (c.code_theme as CodeTheme) || 'github-dark';
      codeScroll = c.code_scroll ?? true;
      autoscroll = c.autoscroll ?? true;
      autoscrollSpeed = c.autoscroll_speed || 50;
      collapseTranscripts = c.collapse_transcripts ?? false;
      collapseLast = c.collapse_last ?? false;
      compactQuick = c.compact_quick ?? false;
      cancelOnResend = c.cancel_on_resend ?? true;
      cancelMode = (c.cancel_mode as CancelMode) || 'drop';
    }
  });

  async function updateSetting(key: string, value: string | number | boolean) {
    try {
      error = null;
      await cfgSet('chat', key, value);
    } catch (e) {
      error = `Ошибка: ${e}`;
      console.error('Chat settings error:', e);
    }
  }

  onMount(() => {
    if (!configStore.config && !configStore.loading) {
      configStore.load();
    }
  });
</script>

<div class="chat-settings p-6 max-w-2xl">
  <h2 class="text-2xl font-semibold mb-2">Чат</h2>
  <p class="text-gray-600 mb-6">
    Внешний вид и поведение ленты сообщений оверлея.
  </p>

  {#if configStore.loading}
    <div class="flex items-center gap-2 text-blue-600">
      <div class="w-4 h-4 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
      <span>Загрузка...</span>
    </div>
  {:else}
    <div class="space-y-6">
      <!-- Message Order -->
      <div>
        <label for="chat-order" class="block font-medium mb-2">Порядок сообщений</label>
        <p class="text-sm text-gray-600 mb-3">Выберите, у какого края чата появляются новые сообщения.</p>
        <select
          id="chat-order"
          bind:value={order}
          onchange={() => updateSetting('order', order)}
          class="w-full px-4 py-2 border border-gray-300 rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option value="bottom">Новые снизу</option>
          <option value="top">Новые сверху</option>
        </select>
      </div>

      <!-- Font Size -->
      <div>
        <label for="font-size" class="block font-medium mb-2">Размер текста сообщений</label>
        <p class="text-sm text-gray-600 mb-3">Изменяет размер основного текста сообщений и расшифровок в чате.</p>
        <div class="flex items-center gap-4">
          <input
            id="font-size"
            type="range"
            min="11"
            max="18"
            step="0.5"
            bind:value={fontSize}
            onchange={() => updateSetting('font_size', fontSize)}
            class="flex-1"
          />
          <span class="w-16 text-right font-medium">{fontSize} px</span>
        </div>
      </div>

      <!-- Code Theme -->
      <div>
        <label for="code-theme" class="block font-medium mb-2">Тема подсветки кода</label>
        <p class="text-sm text-gray-600 mb-3">Выберите тему для блоков кода в ответах и заметках.</p>
        <select
          id="code-theme"
          bind:value={codeTheme}
          onchange={() => updateSetting('code_theme', codeTheme)}
          class="w-full px-4 py-2 border border-gray-300 rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option value="github-dark">GitHub Dark</option>
          <option value="monokai">Monokai</option>
        </select>
      </div>

      <!-- Toggles -->
      <div class="space-y-4">
        <label class="flex items-center gap-3 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={codeScroll}
            onchange={() => updateSetting('code_scroll', codeScroll)}
            class="w-5 h-5 rounded border-gray-300 text-blue-600 focus:ring-2 focus:ring-blue-500"
          />
          <div>
            <div class="font-medium">Независимая прокрутка длинного кода</div>
            <div class="text-sm text-gray-600">Ограничивать высоту длинных блоков кода, прокручивать их отдельно от чата.</div>
          </div>
        </label>

        <label class="flex items-center gap-3 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={autoscroll}
            onchange={() => updateSetting('autoscroll', autoscroll)}
            class="w-5 h-5 rounded border-gray-300 text-blue-600 focus:ring-2 focus:ring-blue-500"
          />
          <div>
            <div class="font-medium">Автопрокрутка чата</div>
            <div class="text-sm text-gray-600">Автоматически следовать за краем новых сообщений.</div>
          </div>
        </label>

        <label class="flex items-center gap-3 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={collapseTranscripts}
            onchange={() => updateSetting('collapse_transcripts', collapseTranscripts)}
            class="w-5 h-5 rounded border-gray-300 text-blue-600 focus:ring-2 focus:ring-blue-500"
          />
          <div>
            <div class="font-medium">Автоматически сворачивать расшифровки</div>
            <div class="text-sm text-gray-600">Группы расшифровок автоматически сворачиваются для компактного отображения.</div>
          </div>
        </label>

        <label class="flex items-center gap-3 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={cancelOnResend}
            onchange={() => updateSetting('cancel_on_resend', cancelOnResend)}
            class="w-5 h-5 rounded border-gray-300 text-blue-600 focus:ring-2 focus:ring-blue-500"
          />
          <div>
            <div class="font-medium">Отменять генерацию при повторной отправке</div>
            <div class="text-sm text-gray-600">Если вы отправляете новое сообщение, пока ИИ ещё отвечает, текущая генерация отменяется.</div>
          </div>
        </label>
      </div>

      {#if error || configStore.error}
        <div class="p-3 bg-red-50 border border-red-200 rounded-lg text-red-800 text-sm">
          {error || configStore.error}
        </div>
      {/if}
    </div>
  {/if}
</div>
