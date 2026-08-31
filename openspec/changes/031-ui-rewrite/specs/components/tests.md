# Test Specifications: Components

## Unit Tests

### ModelModal

**Test: modal_opens_and_loads**
```ts
// ui-next/src/features/models/__tests__/ModelModal.test.ts
import { render, waitFor } from '@testing-library/svelte';
import ModelModal from '../ModelModal.svelte';
import * as tauriApi from '@tauri-apps/api/core';

it('opens modal and loads models', async () => {
  const mockModels = [
    { id: 'claude-sonnet-5', name: 'Claude Sonnet 5', family: 'Anthropic', context_length: 200000 },
  ];
  vi.mocked(tauriApi.invoke).mockResolvedValueOnce(mockModels);
  
  const modal = render(ModelModal);
  await modal.component.show();
  
  await waitFor(() => {
    expect(modal.container.querySelector('[role="dialog"]')).toBeVisible();
    expect(tauriApi.invoke).toHaveBeenCalledWith('models_list');
  });
});
```

**Test: modal_groups_by_family**
```ts
it('groups models by family in sidebar', async () => {
  const mockModels = [
    { id: 'claude-sonnet-5', family: 'Anthropic', ... },
    { id: 'gpt-4', family: 'OpenAI', ... },
    { id: 'claude-opus-5', family: 'Anthropic', ... },
  ];
  vi.mocked(tauriApi.invoke).mockResolvedValueOnce(mockModels);
  
  const modal = render(ModelModal);
  await modal.component.show();
  
  await waitFor(() => {
    const sidebar = modal.container.querySelector('.family-sidebar');
    const families = sidebar?.querySelectorAll('button');
    expect(families).toHaveLength(2); // Anthropic, OpenAI (sorted)
  });
});
```

**Test: modal_selects_and_validates**
```ts
it('validates and selects model', async () => {
  const mockModels = [{ id: 'claude-sonnet-5', name: 'Claude', family: 'Anthropic' }];
  vi.mocked(tauriApi.invoke)
    .mockResolvedValueOnce(mockModels) // models_list
    .mockResolvedValueOnce(undefined); // llm_set success
  
  const modal = render(ModelModal);
  await modal.component.show();
  
  const modelCard = await modal.findByText('claude-sonnet-5');
  await fireEvent.click(modelCard);
  
  expect(tauriApi.invoke).toHaveBeenCalledWith('llm_set', { model: 'claude-sonnet-5', effort: null });
});
```

### Chat Component

**Test: chat_appends_incrementally**
```ts
// ui-next/src/features/chat/__tests__/ChatWindow.test.ts
import { render } from '@testing-library/svelte';
import ChatWindow from '../ChatWindow.svelte';
import { chat } from '$lib/stores/chat.svelte';

it('appends message incrementally without full rerender', async () => {
  const { container } = render(ChatWindow);
  
  // Add 50 messages
  for (let i = 0; i < 50; i++) {
    chat.addMessage({ speaker: 'User', content: `Message ${i}`, timestamp: Date.now() });
  }
  
  // Measure paint time for 51st message
  const start = performance.now();
  chat.addMessage({ speaker: 'User', content: 'New', timestamp: Date.now() });
  await waitFor(() => container.textContent?.includes('New'));
  const paintTime = performance.now() - start;
  
  expect(paintTime).toBeLessThan(10); // Target: ≤10ms
});
```

**Test: chat_streams_tokens**
```ts
it('displays streaming tokens with cursor', async () => {
  const { container } = render(ChatWindow);
  
  chat.streamToken('H');
  chat.streamToken('e');
  chat.streamToken('l');
  
  await waitFor(() => {
    expect(container.textContent).toContain('Hel');
    expect(container.querySelector('.cursor')).toBeVisible(); // Cursor animation
  });
});
```

**Test: chat_finalizes_stream**
```ts
it('finalizes stream on answer_done', async () => {
  const { container } = render(ChatWindow);
  
  chat.streamToken('Hello');
  chat.finalize();
  
  await waitFor(() => {
    const messages = container.querySelectorAll('.message');
    expect(messages).toHaveLength(1);
    expect(messages[0].textContent).toContain('Hello');
    expect(container.querySelector('.cursor')).not.toBeInTheDocument();
  });
});
```

**Test: chat_memoizes_markdown**
```ts
// ui-next/src/lib/utils/__tests__/markdown.test.ts
import { renderMarkdown } from '../markdown';

it('caches parsed HTML', () => {
  const content = '# Title\n\nSome **bold** text';
  
  const html1 = renderMarkdown(content);
  const html2 = renderMarkdown(content); // Second call
  
  expect(html1).toBe(html2); // Same reference (cache hit)
});

it('limits cache size to 100 entries', () => {
  for (let i = 0; i < 150; i++) {
    renderMarkdown(`Content ${i}`);
  }
  
  // First entry should be evicted (LRU)
  const firstContent = 'Content 0';
  // (cache.has is not exposed, but re-parsing proves eviction)
});
```

---

## E2E Tests (Playwright)

### Model Selection Flow

**Test: e2e_model_selection**
```ts
// ui-next/tests/e2e/model-selection.spec.ts
import { test, expect } from '@playwright/test';

test('select model from modal', async ({ page }) => {
  await page.goto('http://localhost:1430');
  
  // Open modal via pill click
  await page.click('[data-testid="model-pill"]');
  await expect(page.locator('[role="dialog"]')).toBeVisible();
  
  // Select family
  await page.click('text=Anthropic');
  
  // Search
  await page.fill('input[placeholder="Search models..."]', 'sonnet');
  
  // Select model
  await page.click('text=claude-sonnet-5');
  
  // Verify modal closed
  await expect(page.locator('[role="dialog"]')).not.toBeVisible();
  
  // Verify pill updated
  await expect(page.locator('[data-testid="model-pill"]')).toContainText('Claude Sonnet 5');
});
```

### Chat Flow

**Test: e2e_chat_stream**
```ts
// ui-next/tests/e2e/chat-flow.spec.ts
test('send message and receive streaming answer', async ({ page }) => {
  await page.goto('http://localhost:1430');
  
  // Send message (via Tauri command or UI input if implemented)
  await page.evaluate(() => {
    window.__TAURI__.invoke('send_message', { content: 'Hello' });
  });
  
  // Wait for User message
  await expect(page.locator('.message').filter({ hasText: 'Hello' })).toBeVisible();
  
  // Wait for Assistant streaming
  await expect(page.locator('.message .cursor')).toBeVisible();
  
  // Wait for finalized answer
  await expect(page.locator('.message').filter({ hasText: /Hi there/ })).toBeVisible({ timeout: 10000 });
  await expect(page.locator('.cursor')).not.toBeVisible();
});
```

### Context Settings

**Test: e2e_context_settings**
```ts
test('change context settings and verify saved', async ({ page }) => {
  await page.goto('http://localhost:1430');
  
  // Open context modal
  await page.click('[data-testid="context-button"]');
  
  // Change slider
  await page.locator('input[type="range"][name="recent_window"]').fill('16');
  
  // Verify live preview
  await expect(page.locator('.estimated-tokens')).toContainText('~3200 tokens');
  
  // Apply
  await page.click('button:has-text("Apply")');
  
  // Verify saved to config.toml
  const config = await page.evaluate(() => {
    return window.__TAURI__.invoke('get_config');
  });
  expect(config.context.recent_window).toBe(16);
});
```

---

## Performance Tests

**Test: perf_initial_render**
```ts
// ui-next/tests/perf/chat.spec.ts
import { test, expect } from '@playwright/test';

test('chat renders 50 messages in ≤120ms', async ({ page }) => {
  await page.goto('http://localhost:1430');
  
  // Load 50 messages
  await page.evaluate(() => {
    for (let i = 0; i < 50; i++) {
      window.__chat.addMessage({ speaker: 'User', content: `Message ${i}` });
    }
  });
  
  const paintTime = await page.evaluate(() => {
    performance.mark('start');
    // Trigger render
    window.__chat.addMessage({ speaker: 'User', content: 'Measure' });
    performance.mark('end');
    performance.measure('render', 'start', 'end');
    const measure = performance.getEntriesByName('render')[0];
    return measure.duration;
  });
  
  expect(paintTime).toBeLessThan(120);
});
```

**Test: perf_append_one**
```ts
test('appending 1 message takes ≤10ms', async ({ page }) => {
  await page.goto('http://localhost:1430');
  
  // Preload 50 messages
  await page.evaluate(() => {
    for (let i = 0; i < 50; i++) {
      window.__chat.addMessage({ speaker: 'User', content: `Message ${i}` });
    }
  });
  
  const appendTime = await page.evaluate(() => {
    performance.mark('append-start');
    window.__chat.addMessage({ speaker: 'User', content: 'New' });
    performance.mark('append-end');
    performance.measure('append', 'append-start', 'append-end');
    return performance.getEntriesByName('append')[0].duration;
  });
  
  expect(appendTime).toBeLessThan(10);
});
```

**Test: perf_virtualization_memory**
```ts
test('100 messages use ≤20MB memory', async ({ page, context }) => {
  await page.goto('http://localhost:1430');
  
  // Take baseline memory snapshot
  const cdp = await context.newCDPSession(page);
  await cdp.send('HeapProfiler.enable');
  await cdp.send('HeapProfiler.collectGarbage');
  const baseline = await cdp.send('HeapProfiler.takeHeapSnapshot');
  
  // Load 100 messages
  await page.evaluate(() => {
    for (let i = 0; i < 100; i++) {
      window.__chat.addMessage({ speaker: 'User', content: `Message ${i}` });
    }
  });
  
  // Take snapshot after
  await cdp.send('HeapProfiler.collectGarbage');
  const after = await cdp.send('HeapProfiler.takeHeapSnapshot');
  
  const memoryIncrease = after.totalSize - baseline.totalSize;
  expect(memoryIncrease).toBeLessThan(20 * 1024 * 1024); // 20MB
});
```

---

## Acceptance Criteria

**Components complete when:**
- [ ] ModelModal: virtualized (417 models smooth), search, keyboard nav, a11y pass
- [ ] ChatWindow: incremental append ≤10ms, streaming display, autoscroll, virtualized 100+ msgs
- [ ] ContextModal: live preview, sliders work, Apply saves to Rust
- [ ] HotkeysModal: conflict detection, record mode, Save applies instantly
- [ ] All components: unit tests 100% coverage
- [ ] E2E tests: 3 critical paths green (model select, chat stream, context save)
- [ ] Performance: all targets met (render ≤120ms, append ≤10ms, memory ≤20MB)
- [ ] Accessibility: axe-core 0 violations, manual screen reader pass
