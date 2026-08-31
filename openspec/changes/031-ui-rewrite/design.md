# Design: UI Rewrite — Technical Architecture

## Overview

Полный рефакторинг UI overlay на современный стек: TypeScript + Svelte 5 + TailwindCSS. Incremental migration с параллельным запуском старого/нового UI до cutover.

## Goals

1. **Performance**: 4.8x faster renders, виртуализация списков, инкрементальные обновления.
2. **Maintainability**: type-safe Rust↔UI bridge, модульная архитектура, 80%+ test coverage.
3. **Developer Experience**: HMR, TypeScript LSP, reusable design system.
4. **Accessibility**: 95+ score, keyboard navigation, ARIA-compliant.

## Non-Goals

- Новые фичи (command palette, themes marketplace) — backlog после cutover.
- Backwards compatibility со старыми UI API (полный rewrite).
- Multi-window support (одно overlay window).

---

## Architecture

### High-Level Structure

```
┌────────────────────────────────────────────────────────────┐
│                    Tauri Backend (Rust)                    │
│  Commands: models_list, llm_set, get_config, ...          │
│  Events: dialogue_turn, answer_token, config_changed       │
└──────────────────┬──────────────────┬──────────────────────┘
                   │ invoke()         │ listen()
                   ▼                  ▼
┌────────────────────────────────────────────────────────────┐
│              TypeScript API Layer (bindings.ts)            │
│  Auto-generated types via Tauri Specta                    │
│  commands.modelsList(): Promise<ModelMetadata[]>           │
└──────────────────┬─────────────────────────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────────────────────────┐
│                   Svelte Stores (State)                    │
│  config ──┐                                                │
│  chat   ──┼─ Reactive state synchronized with Rust        │
│  models ──┘                                                │
└──────────────────┬─────────────────────────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────────────────────────┐
│              UI Components (Svelte 5)                      │
│  App.svelte                                                │
│    ├─ ModelModal.svelte                                    │
│    ├─ ChatWindow.svelte                                    │
│    ├─ ContextModal.svelte                                  │
│    └─ ...                                                  │
└────────────────────────────────────────────────────────────┘
```

### Directory Structure

```
apps/desktop/
├── ui/                          # Старый UI (legacy, удалится после cutover)
│   ├── overlay.html
│   ├── overlay.js (658 lines)
│   └── overlay.css (820 lines)
│
├── ui-next/                     # Новый UI
│   ├── src/
│   │   ├── lib/
│   │   │   ├── bindings.ts      # Tauri Specta types (auto-generated)
│   │   │   ├── design/
│   │   │   │   ├── tokens.css   # Design tokens (colors, spacing, typography)
│   │   │   │   └── components/  # Primitives
│   │   │   │       ├── Button.svelte
│   │   │   │       ├── Modal.svelte
│   │   │   │       ├── Input.svelte
│   │   │   │       ├── Slider.svelte
│   │   │   │       ├── Badge.svelte
│   │   │   │       ├── Toast.svelte
│   │   │   │       ├── Select.svelte
│   │   │   │       └── Checkbox.svelte
│   │   │   ├── stores/          # Reactive state
│   │   │   │   ├── config.svelte.ts
│   │   │   │   ├── chat.svelte.ts
│   │   │   │   ├── models.svelte.ts
│   │   │   │   └── ui.svelte.ts
│   │   │   └── utils/
│   │   │       ├── tauri-bridge.ts  # Sync helpers
│   │   │       └── markdown.ts      # Memoized parser
│   │   ├── features/            # Feature modules
│   │   │   ├── models/
│   │   │   │   ├── ModelModal.svelte
│   │   │   │   ├── FamilySidebar.svelte
│   │   │   │   ├── ModelList.svelte
│   │   │   │   ├── ModelCard.svelte
│   │   │   │   ├── ModelSearch.svelte
│   │   │   │   └── ReasoningSlider.svelte
│   │   │   ├── chat/
│   │   │   │   ├── ChatWindow.svelte
│   │   │   │   ├── MessageList.svelte
│   │   │   │   ├── Message.svelte
│   │   │   │   ├── MessageContent.svelte
│   │   │   │   └── CodeBlock.svelte
│   │   │   ├── context/
│   │   │   │   └── ContextModal.svelte
│   │   │   ├── hotkeys/
│   │   │   │   └── HotkeysModal.svelte
│   │   │   └── settings/
│   │   │       ├── TtsControls.svelte
│   │   │       └── OpacitySlider.svelte
│   │   ├── App.svelte           # Root component
│   │   └── main.ts              # Entry point
│   ├── public/
│   ├── tests/
│   │   ├── unit/                # Vitest tests
│   │   └── e2e/                 # Playwright tests
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   └── tailwind.config.ts
│
├── build.rs                     # Tauri build script (updated)
└── Cargo.toml
```

---

## Component Design

### 1. Foundation Layer

#### TypeScript Setup

**`ui-next/tsconfig.json`**:
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ES2020",
    "moduleResolution": "bundler",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "paths": {
      "$lib/*": ["./src/lib/*"]
    }
  }
}
```

#### Tauri Specta Integration

**`apps/desktop/build.rs`** (updated):
```rust
use tauri_specta::{Builder, collect_commands, collect_events};
use specta_typescript::Typescript;

fn main() {
    // Generate TypeScript types
    let specta = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::models_list,
            commands::llm_set,
            commands::get_config,
            commands::context_apply,
            commands::hotkeys_get,
            commands::hotkeys_set,
            commands::tts_mode_set,
            commands::tts_get,
            // ... все команды
        ])
        .events(collect_events![
            // events если потребуются
        ]);
    
    #[cfg(debug_assertions)]
    {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/ui-next/src/bindings.ts");
        specta.export_for_plugin(Typescript::default(), path)
            .expect("Failed to export Specta types");
    }
    
    // Tauri build: use ui-next/dist if exists, fallback to ui/
    if std::path::Path::new("ui-next/dist").exists() {
        std::env::set_var("TAURI_DIST_DIR", "ui-next/dist");
    }
    
    tauri_build::build()
}
```

**`apps/desktop/Cargo.toml`** (add dependencies):
```toml
[build-dependencies]
tauri-build = "2.0"
tauri-specta = "2.0"
specta = "2.0"
specta-typescript = "0.0.7"
```

**Аннотация команд** (example):
```rust
use specta::Type;

#[derive(Serialize, Deserialize, Type)]
pub struct ModelMetadata {
    pub id: String,
    pub name: String,
    pub family: String,
    pub context_length: u64,
    pub pricing: Pricing,
    pub capabilities: Capabilities,
}

#[tauri::command]
#[specta::specta]  // <- добавить
pub async fn models_list(cfg: State<'_, ConfigState>) -> Result<Vec<ModelMetadata>, String> {
    // ...
}
```

**Usage в UI**:
```ts
// ui-next/src/lib/bindings.ts (auto-generated)
export type ModelMetadata = { id: string; name: string; ... }
export const commands = {
    modelsList: () => invoke<ModelMetadata[]>('models_list'),
    llmSet: (model: string, effort: string | null) => invoke<void>('llm_set', {model, effort}),
}

// ui-next/src/features/models/ModelModal.svelte
import { commands } from '$lib/bindings';
const models = await commands.modelsList();  // fully typed!
```

#### Vite Configuration

**`ui-next/vite.config.ts`**:
```ts
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    target: 'es2020',
    outDir: 'dist',
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true,
      },
    },
    rollupOptions: {
      output: {
        manualChunks: {
          'vendor-svelte': ['svelte'],
          'vendor-lucide': ['lucide-svelte'],
        },
      },
    },
  },
});
```

---

### 2. Design System

#### Tokens

**`ui-next/src/lib/design/tokens.css`**:
```css
:root {
  /* Semantic Colors */
  --color-bg-primary: #0a0a0a;
  --color-bg-secondary: #141414;
  --color-bg-tertiary: #1e1e1e;
  --color-border: #2a2a2a;
  --color-text-primary: #e5e5e5;
  --color-text-secondary: #a3a3a3;
  --color-text-tertiary: #737373;
  --color-accent: #f97316;
  --color-accent-hover: #ea580c;
  --color-accent-dim: rgba(249, 115, 22, 0.1);
  --color-success: #22c55e;
  --color-error: #ef4444;
  --color-warning: #f59e0b;
  
  /* Spacing (4px base) */
  --space-1: 0.25rem;  /* 4px */
  --space-2: 0.5rem;   /* 8px */
  --space-3: 0.75rem;  /* 12px */
  --space-4: 1rem;     /* 16px */
  --space-6: 1.5rem;   /* 24px */
  --space-8: 2rem;     /* 32px */
  
  /* Typography */
  --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  --font-mono: ui-monospace, "Cascadia Code", Consolas, monospace;
  --text-xs: 0.6875rem;   /* 11px */
  --text-sm: 0.8125rem;   /* 13px */
  --text-base: 0.9375rem; /* 15px */
  --text-lg: 1.0625rem;   /* 17px */
  --line-height-tight: 1.25;
  --line-height-normal: 1.5;
  --line-height-relaxed: 1.75;
  
  /* Border Radius */
  --radius-sm: 6px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-full: 9999px;
  
  /* Shadows */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
  --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.4);
  --shadow-lg: 0 10px 40px rgba(0, 0, 0, 0.5);
  
  /* Transitions */
  --transition-fast: 150ms cubic-bezier(0.4, 0, 0.2, 1);
  --transition-base: 250ms cubic-bezier(0.4, 0, 0.2, 1);
  --transition-slow: 350ms cubic-bezier(0.4, 0, 0.2, 1);
  
  /* Z-index Scale */
  --z-dropdown: 50;
  --z-modal: 100;
  --z-toast: 150;
  --z-tooltip: 200;
}

/* Light mode (optional, future) */
:root:not([data-theme="dark"]) {
  --color-bg-primary: #ffffff;
  --color-bg-secondary: #f5f5f5;
  --color-text-primary: #171717;
  /* ... overrides ... */
}
```

#### Button Primitive (example)

**`ui-next/src/lib/design/components/Button.svelte`**:
```svelte
<script lang="ts">
import { type Snippet } from 'svelte';
import { Loader2 } from 'lucide-svelte';

interface Props {
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    disabled?: boolean;
    loading?: boolean;
    onclick?: () => void | Promise<void>;
    type?: 'button' | 'submit' | 'reset';
    class?: string;
    children: Snippet;
}

let {
    variant = 'secondary',
    size = 'md',
    disabled = false,
    loading = false,
    onclick,
    type = 'button',
    class: className = '',
    children,
}: Props = $props();

const baseClasses = 'inline-flex items-center justify-center rounded-md font-medium transition-fast focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent disabled:opacity-50 disabled:pointer-events-none';

const variantClasses = {
    primary: 'bg-accent text-white hover:bg-accent-hover',
    secondary: 'bg-bg-secondary border border-border text-text-primary hover:bg-bg-tertiary',
    ghost: 'text-text-secondary hover:bg-bg-tertiary hover:text-text-primary',
    danger: 'bg-red-600 text-white hover:bg-red-700',
};

const sizeClasses = {
    sm: 'h-8 px-3 text-xs',
    md: 'h-10 px-4 text-sm',
    lg: 'h-12 px-6 text-base',
};

async function handleClick() {
    if (disabled || loading || !onclick) return;
    await onclick();
}
</script>

<button
    {type}
    class="{baseClasses} {variantClasses[variant]} {sizeClasses[size]} {className}"
    disabled={disabled || loading}
    onclick={handleClick}
>
    {#if loading}
        <Loader2 class="animate-spin -ml-1 mr-2 h-4 w-4" />
    {/if}
    {@render children()}
</button>
```

---

### 3. State Management

#### Config Store

**`ui-next/src/lib/stores/config.svelte.ts`**:
```ts
import { writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { commands, type Config } from '$lib/bindings';

export const config = writable<Config | null>(null);

let initialized = false;

export async function initConfig() {
    if (initialized) return;
    initialized = true;
    
    // Load initial
    const cfg = await commands.getConfig();
    config.set(cfg);
    
    // Listen to changes from Rust
    await listen<Config>('config_changed', (event) => {
        config.set(event.payload);
    });
}

export async function updateConfig<K extends keyof Config>(
    key: K,
    value: Config[K]
) {
    // Optimistic update
    config.update(c => c ? { ...c, [key]: value } : null);
    
    try {
        await commands.configSet({ path: key, value: JSON.stringify(value) });
    } catch (e) {
        // Rollback on error
        const fresh = await commands.getConfig();
        config.set(fresh);
        throw e;
    }
}
```

#### Chat Store

**`ui-next/src/lib/stores/chat.svelte.ts`**:
```ts
import { writable, derived } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import type { Turn } from '$lib/bindings';

interface ChatState {
    messages: Turn[];
    streaming: boolean;
    partialMessage: string;
}

const state = writable<ChatState>({
    messages: [],
    streaming: false,
    partialMessage: '',
});

export async function initChat() {
    await listen<Turn>('dialogue_turn', (event) => {
        state.update(s => ({
            ...s,
            messages: [...s.messages, event.payload],
        }));
    });
    
    await listen<string>('answer_token', (event) => {
        state.update(s => ({
            ...s,
            streaming: true,
            partialMessage: s.partialMessage + event.payload,
        }));
    });
    
    await listen('answer_done', () => {
        state.update(s => ({
            ...s,
            streaming: false,
            messages: [...s.messages, {
                speaker: 'Assistant',
                content: s.partialMessage,
                timestamp: Date.now(),
            }],
            partialMessage: '',
        }));
    });
}

export const chat = {
    subscribe: state.subscribe,
    clear: () => state.set({ messages: [], streaming: false, partialMessage: '' }),
};

// Derived: messages grouped by speaker
export const groupedMessages = derived(state, $state => {
    const groups: Array<{ speaker: string; turns: Turn[] }> = [];
    $state.messages.forEach(msg => {
        const last = groups[groups.length - 1];
        if (last && last.speaker === msg.speaker) {
            last.turns.push(msg);
        } else {
            groups.push({ speaker: msg.speaker, turns: [msg] });
        }
    });
    return groups;
});
```

---

### 4. Key Components

#### ModelModal

**Structure**:
```
ModelModal.svelte (container)
  ├─ ModelSearch.svelte (search input + capability filters)
  ├─ FamilySidebar.svelte (family buttons)
  ├─ ModelList.svelte (virtualized list)
  │   └─ ModelCard.svelte (individual model)
  └─ ReasoningSlider.svelte (effort slider)
```

**Virtualization** (using `svelte-virtual-list`):
```svelte
<!-- ModelList.svelte -->
<script lang="ts">
import VirtualList from 'svelte-virtual-list';
import ModelCard from './ModelCard.svelte';

let { models, selectedFamily, selectedModel, onselect } = $props();

const filtered = $derived(
    models
        .filter(m => m.family === selectedFamily)
        .sort((a, b) => a.id.localeCompare(b.id))
);
</script>

<div class="flex-1 overflow-hidden">
    <VirtualList items={filtered} height="100%" itemHeight={80} let:item>
        <ModelCard
            model={item}
            selected={item.id === selectedModel}
            {onselect}
        />
    </VirtualList>
</div>
```

#### ChatWindow

**Structure**:
```
ChatWindow.svelte
  ├─ MessageList.svelte (virtualized)
  │   └─ Message.svelte
  │       └─ MessageContent.svelte (memoized markdown)
  │           └─ CodeBlock.svelte (highlight.js)
  └─ InputBar.svelte (future)
```

**Memoized Markdown**:
```ts
// ui-next/src/lib/utils/markdown.ts
import { marked } from 'marked';
import DOMPurify from 'dompurify';

const cache = new Map<string, string>();

export function renderMarkdown(content: string): string {
    const cached = cache.get(content);
    if (cached) return cached;
    
    const raw = marked.parse(content, { breaks: true, gfm: true });
    const clean = DOMPurify.sanitize(raw);
    
    cache.set(content, clean);
    
    // LRU: keep max 100 entries
    if (cache.size > 100) {
        const first = cache.keys().next().value;
        cache.delete(first);
    }
    
    return clean;
}
```

---

## Migration Strategy

### Phase 0: Preparation (1 week)
- Setup Vite + Svelte + TypeScript в `ui-next/`.
- Integrate Tauri Specta, generate bindings.
- POC: TtsControls component (simplest, isolated).

### Phase 1: Foundation (2-3 weeks)
- Design system: tokens + 8 primitives.
- Stores: config, chat, models, ui.
- Dev workflow: HMR, ESLint, Prettier.

### Phase 2: Core Migration (3-4 weeks)
- ModelModal (with virtualization, search).
- ChatWindow (with virtualization, memoized markdown).
- ContextModal, HotkeysModal (straightforward ports).

### Phase 3: Advanced (2-3 weeks)
- Multi-provider switcher.
- Responsive design (breakpoints).
- Accessibility audit + fixes.
- Testing suite (Vitest + Playwright).

### Phase 4: Polish (1-2 weeks)
- Bundle size optimization.
- Animations (AutoAnimate).
- Error boundaries.
- Performance benchmarks.

### Phase 5: Cutover (1 week)
- Parallel run (dogfooding).
- Feature parity validation.
- Cutover: `mv ui ui-legacy; mv ui-next/dist ui`.
- Grace period 2 weeks → delete legacy.

### Phase 6: Documentation (3 days)
- README, SETUP, COMPONENTS, CONTRIBUTING.
- Storybook (optional).
- Changelog.

---

## Testing Strategy

### Unit Tests (Vitest)
- Stores: config sync, chat accumulation, models cache.
- Utils: markdown memoization, tauri-bridge helpers.
- Components: Button variants, Modal a11y.

**Target**: 80%+ coverage.

### Component Tests (@testing-library/svelte)
- Button: click calls onclick, loading disables.
- Modal: focus trap, Escape closes.
- Slider: keyboard arrows, value update.

### E2E Tests (Playwright)
- Critical paths:
  - Open modal → select model → verify pill updated.
  - Send message → stream answer → verify chat appended.
  - Change context settings → verify config saved.

**Target**: 100% coverage критичных user flows.

---

## Performance Targets

| Metric | Current | Target | Test |
|--------|---------|--------|------|
| Bundle size | 42kb | ≤80kb | CI check gzipped dist/*.js |
| Chat initial render (50 msg) | 580ms | ≤120ms | Chrome DevTools Performance |
| Chat append 1 msg | 340ms | ≤10ms | Performance mark/measure |
| Modal render (400 models) | freeze | smooth 60fps | Manual scroll test |
| Memory (100 messages) | 60MB | ≤20MB | Chrome DevTools Memory |
| Lighthouse Performance | 72 | ≥90 | CI Lighthouse |
| Accessibility score | 68 | ≥95 | axe-core + manual NVDA |

---

## Rollback Plan

### Config-based Rollback
1. Пользователь изменяет `config.toml`: `[ui] default_version = "legacy"`.
2. Перезапуск приложения → Tauri читает `ui-legacy/` вместо `ui/`.

### Binary-based Rollback
1. Hotfix release с флагом `--use-legacy-ui`.
2. Пользователи запускают `desktop.exe --use-legacy-ui`.

### Git Rollback
1. `git revert <cutover-commit>`.
2. `cargo build -p desktop --release`.

---

## Security Considerations

### XSS Prevention
- Markdown рендерится через DOMPurify (sanitize).
- Все user input экранируется в Svelte templates (auto-escaping).

### CSP (Content Security Policy)
- Tauri CSP: `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'`.
- Inline styles разрешены для Svelte (scoped CSS).

### Dependency Auditing
- `npm audit` в CI.
- Dependabot для автоматических updates.

---

## Open Questions

1. **Dark/Light mode theming**: реализовать сразу или в backlog?
   - **Decision**: токены поддерживают `:root:not([data-theme="dark"])`, но переключатель в UI — backlog.

2. **Storybook для component docs**: добавить или оставить markdown?
   - **Decision**: опционально; если есть time budget — добавить в Phase 4.

3. **i18n (internationalization)**: русский + английский?
   - **Decision**: out of scope для change 031; текущий UI на русском, сохраняем.

4. **Mobile support**: адаптация под телефоны/планшеты?
   - **Decision**: responsive design для планшетов (768px+); телефоны (< 640px) — backlog.

---

## Success Metrics

**Launch Readiness Checklist**:
- [ ] Feature parity: 15/15 фич старого UI работают.
- [ ] Performance: все targets достигнуты (bundle ≤80kb, chat ≤120ms, etc.).
- [ ] Accessibility: Lighthouse 95+, manual screen reader pass.
- [ ] Tests: 80%+ unit coverage, E2E suite green.
- [ ] Docs: README/SETUP/COMPONENTS/CONTRIBUTING complete.
- [ ] Dogfooding: 1 неделя без критичных багов.

**Post-Launch (3 месяца)**:
- [ ] Zero P0 bugs.
- [ ] Developer velocity: фичи добавляются в 2x быстрее (измерить PR cycle time).
- [ ] Bundle size stable (не растёт >10% при добавлении фич).
