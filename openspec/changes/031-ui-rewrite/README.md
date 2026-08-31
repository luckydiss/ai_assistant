# Change 031: UI Rewrite — Full Stack Modernization

**Status**: PLANNED  
**Timeline**: 10-16 weeks (2.5-4 months)  
**Impact**: Complete UI rewrite on modern stack (TypeScript + Svelte 5 + Vite + TailwindCSS)

---

## Quick Links

### Planning Documents
- **[Proposal](./proposal.md)** — WHY: current pain points, goals, metrics, risks
- **[Design](./design.md)** — HOW: architecture, stack choices, migration strategy
- **[Tasks](./tasks.md)** — WHAT: detailed checklist (6 phases, 100+ tasks)

### Specifications (Detailed Requirements)

#### Foundation
- **[Foundation Spec](./specs/foundation/spec.md)** — TypeScript, Tauri Specta, Vite, build system
- **[Foundation Tests](./specs/foundation/tests.md)** — Unit/integration tests for build pipeline

#### Design System
- **[Design System Spec](./specs/design-system/spec.md)** — Tokens, primitives (Button, Modal, Input, Slider, etc.), accessibility
- **[Design System Tests](./specs/design-system/tests.md)** — Component tests, a11y validation, visual regression

#### State Management
- **[State Spec](./specs/state/spec.md)** — Svelte stores (config, chat, models, ui), Tauri event bridge
- **[State Tests](./specs/state/tests.md)** — Store sync tests, optimistic updates, cache validation

#### Components
- **[Model Modal Spec](./specs/components/model-modal-spec.md)** — ModelModal virtualization, search, keyboard nav
- **[Chat Spec](./specs/components/chat-spec.md)** — Chat incremental render, streaming, markdown memoization
- **[Component Tests](./specs/components/tests.md)** — Unit/E2E/performance tests for all components

#### Migration
- **[Migration Spec](./specs/migration/spec.md)** — Parallel deployment, cutover strategy, rollback plan
- **[Migration Tests](./specs/migration/tests.md)** — Feature parity validation, regression detection, grace period

---

## Key Metrics

### Current (Baseline)
| Metric | Value |
|--------|-------|
| Bundle size | 42kb |
| Chat render (50 msg) | 580ms |
| Chat append (1 msg) | 340ms |
| Accessibility | 68/100 |
| Test coverage | 0% |

### Target (After Rewrite)
| Metric | Target | Improvement |
|--------|--------|-------------|
| Bundle size | ≤80kb | Acceptable growth |
| Chat render (50 msg) | ≤120ms | **4.8x faster** |
| Chat append (1 msg) | ≤10ms | **34x faster** |
| Accessibility | ≥95/100 | **+27 points** |
| Test coverage | 80%+ | **∞** |

---

## Technology Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| **Language** | TypeScript 5.7 | Type safety, LSP, refactor confidence |
| **UI Framework** | Svelte 5 | Smallest bundle (compile-to-vanilla), reactive runes API |
| **Build** | Vite 6 | HMR, fast, Tauri integration |
| **Styling** | TailwindCSS 4 + CSS variables | Utility-first, design tokens, no magic numbers |
| **State** | Svelte stores + Tauri events | Reactive, bidirectional Rust↔UI sync |
| **Testing** | Vitest + Playwright | Unit + E2E, fast, Vite-native |
| **Icons** | Lucide Svelte | Tree-shakeable, 1kb/icon |

---

## Migration Phases

### Phase 0: Preparation (1 week)
- Audit current UI
- POC: TtsControls.svelte
- Baseline metrics

### Phase 1: Foundation (2-3 weeks)
- TypeScript + Tauri Specta types
- Design system (tokens + 8 primitives)
- Stores (config, chat, models, ui)

### Phase 2: Core Migration (3-4 weeks)
- ModelModal (virtualized, search)
- ChatWindow (incremental render, streaming)
- ContextModal, HotkeysModal
- TTS/Screenshot/Settings controls

### Phase 3: Advanced (2-3 weeks)
- Multi-provider switcher
- Responsive design
- Accessibility audit (95+ score)
- Testing suite (80%+ coverage)

### Phase 4: Polish (1-2 weeks)
- Bundle optimization (≤80kb)
- Animations (AutoAnimate)
- Error boundaries
- Performance benchmarks

### Phase 5: Cutover (1 week)
- Parallel run (dogfooding)
- Feature parity validation
- Atomic cutover: `mv ui ui-legacy; mv ui-next/dist ui`
- Grace period 2 weeks

### Phase 6: Documentation (3 days)
- README, SETUP, COMPONENTS, CONTRIBUTING
- Changelog, release notes

---

## Success Criteria

**Launch readiness** (all must be ✅):
- [ ] Feature parity: 15/15 features work identically
- [ ] Performance: all targets met (bundle, render, memory)
- [ ] Accessibility: Lighthouse ≥95, screen reader pass
- [ ] Tests: 80%+ unit coverage, E2E suite green
- [ ] Docs: README/SETUP/COMPONENTS/CONTRIBUTING complete
- [ ] Dogfooding: 1 week without P0 bugs

**Post-launch** (3 months):
- [ ] Zero critical regressions
- [ ] Developer velocity: features ship 2x faster
- [ ] Bundle size stable (no >10% growth)

---

## Rollback Plan

If critical bug after cutover:
1. **Config-based**: `[ui] default_version = "legacy"` → restart
2. **Git revert**: `git revert <cutover-commit>; cargo build`
3. **Hotfix binary**: `--use-legacy-ui` flag

Grace period: **2 weeks**. After 0 P0 bugs → delete `ui-legacy/`.

---

## Directory Structure

```
openspec/changes/031-ui-rewrite/
├── proposal.md           # WHY: pain points, goals, metrics
├── design.md             # HOW: architecture, stack, strategy
├── tasks.md              # WHAT: 6-phase checklist (100+ tasks)
├── README.md             # This file (navigation index)
└── specs/                # Detailed requirements
    ├── foundation/
    │   ├── spec.md       # TypeScript, build, Tauri Specta
    │   └── tests.md      # Build pipeline tests
    ├── design-system/
    │   ├── spec.md       # Tokens, primitives, a11y
    │   └── tests.md      # Component/visual/a11y tests
    ├── state/
    │   ├── spec.md       # Stores, Tauri bridge
    │   └── tests.md      # Sync/cache tests
    ├── components/
    │   ├── model-modal-spec.md
    │   ├── chat-spec.md
    │   └── tests.md      # Unit/E2E/perf tests
    └── migration/
        ├── spec.md       # Cutover strategy, rollback
        └── tests.md      # Feature parity, regression detection
```

---

## Contributing

This change is **not yet started**. Once Phase 0 begins:
1. Read [proposal.md](./proposal.md) for context
2. Check [tasks.md](./tasks.md) for current phase
3. Pick unchecked task, create branch `031-ui/<task-id>`
4. Implement per [specs/](./specs/)
5. Run tests: `npm run lint && npm run type-check && npm test`
6. Submit PR with task checkbox update in `tasks.md`

---

## Questions?

- **Why Svelte over React/Vue?** See [proposal.md §"Стек"](./proposal.md#стек-обоснование-выбора)
- **Why not keep vanilla JS?** See [proposal.md §"Текущие боли"](./proposal.md#текущие-боли)
- **What if we find a blocker mid-migration?** Incremental design allows stopping at any phase; old UI remains functional.
- **Timeline too long?** Phases can be parallelized if team grows; adjust per [tasks.md](./tasks.md).

---

## Related Changes

- **Change 030 (Models)**: Models catalog already migrated to metadata-driven; UI rewrite consumes this data.
- **Future (Post-031)**: Command palette, themes marketplace, chat export (backlog).

---

**Last Updated**: 2024-01-08  
**Owner**: TBD  
**Reviewers**: TBD
