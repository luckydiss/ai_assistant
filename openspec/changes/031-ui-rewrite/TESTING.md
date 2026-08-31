# Performance & Accessibility Testing Checklist

## Task 0.7: Performance Testing (Manual)

### Build Metrics ✅
```
Bundle size: 77.93 KB (production)
├── CSS:  22.30 KB (gzip: 6.12 KB)
└── JS:   55.63 KB (gzip: 21.78 KB)

Build time:  312ms (cold)
Type check:  0 errors, 0 warnings
```

### Manual Testing Checklist
Run these tests when POC is integrated with Tauri desktop app:

- [ ] **First Paint**: Measure time to first visual render
  - Target: ≤ 500ms
  - Method: DevTools Performance panel

- [ ] **Time to Interactive (TTI)**: Measure when app becomes fully interactive
  - Target: ≤ 1500ms
  - Method: DevTools Lighthouse or manual stopwatch

- [ ] **Memory Usage**: Check baseline memory footprint
  - Target: ≤ 150 MB (Tauri + Webview + app)
  - Method: Task Manager / Process Explorer

- [ ] **HMR Performance**: Measure hot reload time during development
  - Target: ≤ 100ms
  - Method: Edit TtsControls.svelte, measure console timestamp

- [ ] **TTS Controls Load**: Measure component mount time
  - Target: ≤ 50ms
  - Method: Performance.mark() in onMount

- [ ] **Tauri Command Latency**: Measure `invoke('get_config')` roundtrip
  - Target: ≤ 10ms
  - Method: console.time() in TtsControls

### Automated Testing (Phase 1)
When CI/CD is set up:
```bash
# Lighthouse CI
npm run lighthouse -- --preset=desktop --quiet

# Bundle size monitoring
npm run build && size-limit

# Performance budgets (vite-plugin-checker)
vite build --mode=production
```

---

## Task 0.8: Accessibility Testing (Manual)

### Component-Level Checks (TtsControls.svelte) ✅

#### Keyboard Navigation
- [ ] **Tab order**: Focus moves logically (label → select → buttons)
- [ ] **Enter/Space**: Activate buttons with keyboard
- [ ] **Arrow keys**: Navigate select dropdown
- [ ] **Escape**: Close select dropdown

#### Screen Reader
- [ ] **Labels**: `<label for="tts-mode">` properly associates with `<select>`
- [ ] **Error messages**: Announced when displayed
- [ ] **Loading state**: "Загрузка..." announced
- [ ] **Status changes**: Mode change announced

#### Color Contrast
- [ ] **Normal text**: ≥ 4.5:1 (gray-600 on white)
- [ ] **Headings**: ≥ 4.5:1 (gray-900 on white)
- [ ] **Error text**: ≥ 4.5:1 (red-800 on red-50)
- [ ] **Focus indicators**: ≥ 3:1 (blue-500 ring)

#### ARIA
- [ ] **No redundant ARIA**: Native HTML elements used (`<select>`, `<label>`)
- [ ] **Live regions**: Error messages in polite live region (implicit)
- [ ] **Disabled states**: `disabled` attribute on select during loading

### Page-Level Checks (App.svelte)

- [ ] **Document title**: `<title>` element present in index.html
- [ ] **Lang attribute**: `<html lang="ru">` (or appropriate language)
- [ ] **Landmark regions**: `<main>` wraps primary content
- [ ] **Heading hierarchy**: h1 → h2 (no skips)
- [ ] **Skip link**: "Skip to main content" for keyboard users (optional for POC)

### Automated Testing (Phase 1)
```bash
# axe-core CLI (requires running server)
npm run axe

# Playwright + axe
npx playwright test --grep @a11y

# Svelte a11y compiler warnings
npm run check  # Already passing ✅
```

---

## Known Issues / Deferred

### TailwindCSS v4 Warnings (Non-blocking)
```
[lightningcss minify] Unknown at rule: @theme
[lightningcss minify] Unknown at rule: @tailwind
```
- **Impact**: None (CSS compiles correctly)
- **Resolution**: Wait for Tailwind v4 stable or switch minifier

### Lighthouse Automation Blocked
```
Runtime error: Chrome prevented page load with an interstitial
```
- **Cause**: Vite preview server + headless Chrome incompatibility
- **Workaround**: Manual testing with DevTools when integrated with Tauri
- **Resolution**: Phase 1 — CI with real Tauri build

---

## Test Execution Plan

### Phase 0 (POC) — Manual Only
1. Visual inspection of TtsControls keyboard nav
2. Check TailwindCSS contrast with DevTools color picker
3. Verify svelte-check passes (✅ done)

### Phase 1 (Foundation) — CI Integration
1. GitHub Actions: `npm run lighthouse` on PR
2. Pre-commit hook: `npm run check` (type + a11y)
3. Playwright E2E: keyboard nav + screen reader tests

### Phase 2 (Core Components) — Full Automation
1. Visual regression: Percy / Chromatic
2. Performance budgets: fail build if bundle > 100 KB
3. A11y gate: fail if axe violations ≥ 1

---

## Acceptance Criteria

### Performance (Target ≥95 Lighthouse Score)
- [x] Bundle size ≤ 80 KB ✅ (77.93 KB)
- [ ] First Paint ≤ 500ms (manual test required)
- [ ] TTI ≤ 1500ms (manual test required)
- [ ] Memory ≤ 150 MB (manual test required)

### Accessibility (Target 100% Pass)
- [x] Svelte compiler a11y warnings: 0 ✅
- [ ] axe-core violations: 0 (blocked by server issue)
- [ ] Keyboard nav: 100% functional (manual test required)
- [ ] Screen reader: 100% announced (manual test required)

---

## Next Steps

1. **Integrate POC with Tauri**: Update `tauri.conf.json` to point to `ui-next/dist`
2. **Manual Testing**: Run checklist in actual desktop app
3. **Document Results**: Update POC.md with manual test results
4. **Phase 1 Approval**: If tests pass, proceed to Foundation tasks

---

**Status**: ⏳ Awaiting Tauri integration for manual testing  
**Blocker**: Lighthouse automation requires running Tauri app (not standalone Vite server)  
**Recommendation**: Approve POC based on bundle size + type safety; defer full perf/a11y audit to Phase 1
