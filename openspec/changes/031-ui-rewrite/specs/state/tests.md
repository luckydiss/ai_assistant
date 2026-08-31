# Test Specifications: State Management

## Unit Tests

### Config Store

**Test: config_store_loads_initial**
```ts
// ui-next/src/lib/stores/__tests__/config.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { config, initConfig } from '../config.svelte';
import * as tauriApi from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core');

describe('config store', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('loads initial config', async () => {
    const mockConfig = { llm: { provider: 'openrouter', model: 'claude-sonnet-5' } };
    vi.mocked(tauriApi.invoke).mockResolvedValueOnce(mockConfig);
    
    await initConfig();
    
    expect(get(config)).toEqual(mockConfig);
    expect(tauriApi.invoke).toHaveBeenCalledWith('get_config');
  });
});
```

**Test: config_store_reacts_to_rust_change**
```ts
import { listen } from '@tauri-apps/api/event';

it('updates on config_changed event', async () => {
  const initialConfig = { llm: { provider: 'dslab' } };
  vi.mocked(tauriApi.invoke).mockResolvedValueOnce(initialConfig);
  
  let eventCallback: (e: any) => void;
  vi.mocked(listen).mockImplementation(async (event, cb) => {
    if (event === 'config_changed') eventCallback = cb;
    return () => {};
  });
  
  await initConfig();
  expect(get(config)?.llm.provider).toBe('dslab');
  
  // Simulate event from Rust
  const newConfig = { llm: { provider: 'openrouter' } };
  eventCallback!({ payload: newConfig });
  
  expect(get(config)?.llm.provider).toBe('openrouter');
});
```

**Test: config_store_optimistic**
```ts
import { updateConfig } from '../config.svelte';

it('applies optimistic update, rollback on error', async () => {
  const initialConfig = { ui: { opacity: 85 } };
  vi.mocked(tauriApi.invoke).mockResolvedValueOnce(initialConfig);
  await initConfig();
  
  // Mock configSet to fail
  vi.mocked(tauriApi.invoke).mockRejectedValueOnce(new Error('validation failed'));
  
  // Optimistic update
  const promise = updateConfig('ui', { opacity: 90 });
  
  // Should update immediately
  expect(get(config)?.ui.opacity).toBe(90);
  
  // After rejection, should rollback
  await expect(promise).rejects.toThrow('validation failed');
  
  // Rollback: fetch fresh config
  vi.mocked(tauriApi.invoke).mockResolvedValueOnce(initialConfig);
  // (rollback logic in implementation)
  expect(get(config)?.ui.opacity).toBe(85);
});
```

---

### Chat Store

**Test: chat_store_turn**
```ts
// ui-next/src/lib/stores/__tests__/chat.test.ts
import { get } from 'svelte/store';
import { chat, initChat } from '../chat.svelte';
import { listen } from '@tauri-apps/api/event';

it('appends turn on dialogue_turn event', async () => {
  let turnCallback: (e: any) => void;
  vi.mocked(listen).mockImplementation(async (event, cb) => {
    if (event === 'dialogue_turn') turnCallback = cb;
    return () => {};
  });
  
  await initChat();
  
  const turn = { speaker: 'User', content: 'Hello', timestamp: Date.now() };
  turnCallback!({ payload: turn });
  
  const state = get(chat);
  expect(state.messages).toHaveLength(1);
  expect(state.messages[0]).toEqual(turn);
});
```

**Test: chat_store_streaming**
```ts
it('accumulates streaming tokens', async () => {
  let tokenCallback: (e: any) => void;
  vi.mocked(listen).mockImplementation(async (event, cb) => {
    if (event === 'answer_token') tokenCallback = cb;
    return () => {};
  });
  
  await initChat();
  
  tokenCallback!({ payload: 'H' });
  tokenCallback!({ payload: 'e' });
  tokenCallback!({ payload: 'l' });
  
  const state = get(chat);
  expect(state.streaming).toBe(true);
  expect(state.partialMessage).toBe('Hel');
});
```

**Test: chat_store_stream_done**
```ts
it('finalizes stream on answer_done', async () => {
  let tokenCallback: (e: any) => void;
  let doneCallback: () => void;
  vi.mocked(listen).mockImplementation(async (event, cb) => {
    if (event === 'answer_token') tokenCallback = cb;
    if (event === 'answer_done') doneCallback = cb;
    return () => {};
  });
  
  await initChat();
  
  tokenCallback!({ payload: 'Hello' });
  doneCallback!();
  
  const state = get(chat);
  expect(state.streaming).toBe(false);
  expect(state.partialMessage).toBe('');
  expect(state.messages).toHaveLength(1);
  expect(state.messages[0].content).toBe('Hello');
  expect(state.messages[0].speaker).toBe('Assistant');
});
```

**Test: chat_grouped_messages** (derived store)
```ts
import { groupedMessages } from '../chat.svelte';

it('groups consecutive same-speaker turns', () => {
  // Setup chat with 4 turns: User, User, Assistant, Assistant
  const turns = [
    { speaker: 'User', content: 'A' },
    { speaker: 'User', content: 'B' },
    { speaker: 'Assistant', content: 'C' },
    { speaker: 'Assistant', content: 'D' },
  ];
  
  // (simulate adding turns via events)
  
  const groups = get(groupedMessages);
  expect(groups).toHaveLength(2);
  expect(groups[0].speaker).toBe('User');
  expect(groups[0].turns).toHaveLength(2);
  expect(groups[1].speaker).toBe('Assistant');
  expect(groups[1].turns).toHaveLength(2);
});
```

---

### Models Store

**Test: models_store_lazy_load**
```ts
// ui-next/src/lib/stores/__tests__/models.test.ts
import { loadModels } from '../models.svelte';

it('loads models on first access', async () => {
  const mockModels = [{ id: 'claude-sonnet-5', name: 'Claude Sonnet 5', family: 'Anthropic' }];
  vi.mocked(tauriApi.invoke).mockResolvedValueOnce(mockModels);
  
  const result = await loadModels();
  
  expect(result).toEqual(mockModels);
  expect(tauriApi.invoke).toHaveBeenCalledWith('models_list');
});
```

**Test: models_store_cache**
```ts
it('returns cached models within 5 minutes', async () => {
  const mockModels = [{ id: 'test', name: 'Test', family: 'Test' }];
  vi.mocked(tauriApi.invoke).mockResolvedValueOnce(mockModels);
  
  await loadModels(); // First call
  vi.mocked(tauriApi.invoke).mockClear();
  
  await loadModels(); // Second call within cache window
  
  expect(tauriApi.invoke).not.toHaveBeenCalled(); // Cache hit
});
```

**Test: models_store_cache_expire**
```ts
import { vi } from 'vitest';

it('refetches after cache expiry', async () => {
  const mockModels = [{ id: 'test', name: 'Test', family: 'Test' }];
  vi.mocked(tauriApi.invoke).mockResolvedValue(mockModels);
  
  await loadModels();
  
  // Advance time 6 minutes
  vi.useFakeTimers();
  vi.advanceTimersByTime(6 * 60 * 1000);
  vi.useRealTimers();
  
  vi.mocked(tauriApi.invoke).mockClear();
  await loadModels();
  
  expect(tauriApi.invoke).toHaveBeenCalledWith('models_list'); // Cache miss
});
```

---

### UI Store

**Test: ui_store_toast_queue**
```ts
// ui-next/src/lib/stores/__tests__/ui.test.ts
import { get } from 'svelte/store';
import { toast, toastQueue } from '../ui.svelte';

it('queues toasts and shows max 3', () => {
  toast.success('A');
  toast.error('B');
  toast.info('C');
  toast.success('D');
  toast.success('E');
  
  const queue = get(toastQueue);
  expect(queue.visible).toHaveLength(3); // A, B, C
  expect(queue.pending).toHaveLength(2); // D, E
});

it('auto-dismisses toasts after timeout', async () => {
  vi.useFakeTimers();
  
  toast.success('Test'); // 3s timeout
  expect(get(toastQueue).visible).toHaveLength(1);
  
  vi.advanceTimersByTime(3000);
  expect(get(toastQueue).visible).toHaveLength(0);
  
  vi.useRealTimers();
});
```

**Test: ui_store_modal_stack**
```ts
import { modalStack, pushModal, popModal } from '../ui.svelte';

it('manages modal stack for nested modals', () => {
  pushModal('model-modal');
  pushModal('context-modal');
  
  expect(get(modalStack)).toEqual(['model-modal', 'context-modal']);
  
  popModal(); // Escape pressed
  expect(get(modalStack)).toEqual(['model-modal']);
});
```

---

## Integration Tests

### Type-Safe Access

**Test: store_type_check**
```bash
# Create temp file with type error
cat > ui-next/src/__tests__/type-error-temp.ts << EOF
import { config } from '\$lib/stores/config.svelte';
import { get } from 'svelte/store';

const cfg = get(config);
cfg.llm.provider = 123; // Type error: number not assignable to string
EOF

npx tsc --noEmit
# Expected: compilation error
```

---

## Acceptance Criteria

**State Management complete when:**
- [ ] Config store: syncs bidirectionally with Rust (load, events, optimistic updates)
- [ ] Chat store: handles streaming (token accumulation, finalization)
- [ ] Models store: lazy loading + 5-minute cache
- [ ] UI store: toast queue (max 3), modal stack (Escape closes top)
- [ ] All stores: 100% unit test coverage
- [ ] Type safety: TypeScript catches misuse at compile time
- [ ] Integration: manual test with running Tauri app confirms stores update on Rust events
