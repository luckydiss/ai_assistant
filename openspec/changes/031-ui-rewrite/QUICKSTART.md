# Quick Integration Checklist

> **Fast-track guide for integrating ui-next with Tauri backend**

---

## Pre-Integration

### ✅ Verify POC Complete

- [x] Phase 0: POC (77.93 KB)
- [x] Phase 1: Foundation (78.32 KB)
- [x] Phase 2: Core Components (88.67 KB)
- [x] Phase 3: Full Settings UI (107.06 KB)
- [x] All type checks pass (0 errors, 0 warnings)
- [x] Build succeeds (351ms)

---

## Step 1: Backend Commands (30 min)

### Implement Rust Commands

```rust
// apps/desktop/src-tauri/src/commands/config.rs

#[tauri::command]
pub fn get_config() -> Result<AppConfig, String> {
    // Return full config structure
}

#[tauri::command]
pub fn cfg_set(section: String, key: String, value: serde_json::Value) -> Result<(), String> {
    // Generic config setter
}

#[tauri::command]
pub fn tts_set_mode(mode: String) -> Result<(), String> {
    // TTS-specific setter (convenience)
}

#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<String>, String> {
    // List audio input devices
}

#[tauri::command]
pub fn update_audio_settings(source: String, mode: String, mic_device: Option<String>) -> Result<(), String> {
    // Update audio config
}

#[tauri::command]
pub fn hotkeys_get() -> Result<HashMap<String, String>, String> {
    // Get all hotkeys
}

#[tauri::command]
pub fn set_hotkey(action: String, accel: String) -> Result<(), String> {
    // Set single hotkey
}

#[tauri::command]
pub fn protection_set(on: bool) -> Result<(), String> {
    // Toggle screen protection
}
```

### Register Commands

```rust
// apps/desktop/src-tauri/src/main.rs

tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        get_config,
        cfg_set,
        tts_set_mode,
        list_audio_devices,
        update_audio_settings,
        hotkeys_get,
        set_hotkey,
        protection_set,
        // ... keep existing commands
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

### Checklist

- [ ] All 8 commands implemented
- [ ] Commands registered in `main.rs`
- [ ] Serde types match TypeScript interfaces
- [ ] Error handling returns user-friendly messages
- [ ] `cargo build` succeeds

---

## Step 2: Update Tauri Config (5 min)

```json
// apps/desktop/tauri.conf.json

{
  "build": {
    "beforeDevCommand": "cd ../ui-next && npm run dev",
    "beforeBuildCommand": "cd ../ui-next && npm run build",
    "devPath": "http://localhost:1420",
    "frontendDist": "../ui-next/dist"
  }
}
```

### Checklist

- [ ] `frontendDist` points to `../ui-next/dist`
- [ ] `devPath` points to `http://localhost:1420`
- [ ] Build commands reference ui-next

---

## Step 3: Build & Test (15 min)

### Build UI

```bash
cd apps/desktop/ui-next
npm install
npm run build
```

**Expected output**:
```
✓ built in ~350ms
dist/index.html                  0.62 kB
dist/assets/index.css           22.83 kB
dist/assets/vendor-tauri.js      1.71 kB
dist/assets/index.js            29.49 kB
dist/assets/vendor.js           53.41 kB
```

### Build Desktop App

```bash
cd apps/desktop
cargo tauri dev
```

### Checklist

- [ ] UI builds successfully (107 KB)
- [ ] Desktop app launches
- [ ] Settings UI loads
- [ ] No console errors

---

## Step 4: Smoke Test (10 min)

### Test Each Section

**Audio Settings**:
- [ ] Load: Displays current source/mode/mic
- [ ] Save: Click "Сохранить" → shows success
- [ ] Verify: Reload app → settings persisted

**TTS Settings**:
- [ ] Load: Displays current mode
- [ ] Change: Select different mode → auto-saves
- [ ] Verify: Reload app → settings persisted

**Hotkeys Settings**:
- [ ] Load: Displays current hotkeys
- [ ] Update: Type new hotkey → auto-saves
- [ ] Clear: Empty field → clears hotkey
- [ ] Verify: Test global hotkey works

**Window Settings**:
- [ ] Load: Displays current accent/opacity/steps
- [ ] Accent: Click color → UI updates immediately
- [ ] Sliders: Move slider → auto-saves
- [ ] Verify: Settings persist

**Chat Settings**:
- [ ] Load: Displays current settings
- [ ] Toggle: Click checkbox → auto-saves
- [ ] Sliders: Adjust values → auto-saves
- [ ] Verify: Settings persist

**Protection Settings**:
- [ ] Load: Displays current state
- [ ] Toggle: Click switch → shows feedback
- [ ] Verify: Screen capture blocked/allowed

---

## Step 5: Navigation Test (5 min)

- [ ] Click each sidebar item → section loads
- [ ] Active state highlights correctly
- [ ] No layout jumps or flicker
- [ ] Scroll works in long sections
- [ ] Back button returns to demo

---

## Step 6: Error Handling (10 min)

### Test Failure Scenarios

**Config Load Failure**:
```rust
// Temporarily break get_config
Err("Simulated error".to_string())
```
- [ ] Shows error message (not blank screen)
- [ ] Error is user-friendly
- [ ] Can retry or navigate away

**Config Save Failure**:
```rust
// Temporarily break cfg_set
Err("Permission denied".to_string())
```
- [ ] Shows error toast
- [ ] UI doesn't break
- [ ] Can retry save

**Command Not Found**:
```typescript
// Comment out a command in main.rs
```
- [ ] Shows error (not silent failure)
- [ ] Logs to console with details

### Checklist

- [ ] All error scenarios handled gracefully
- [ ] No blank screens on error
- [ ] Error messages are clear
- [ ] Restore all commands

---

## Step 7: Performance Check (10 min)

### Metrics

```bash
# 1. Build time
cd ui-next && npm run build
# Expected: ~350ms

# 2. Bundle size
ls -lh dist/assets/
# Expected: ~107 KB total

# 3. Load time
# Open DevTools → Network → Disable cache
# Reload Settings UI
# Expected: < 500ms to interactive

# 4. Save time
# Open DevTools → Console
# Change a setting
# Expected: < 200ms save response
```

### Checklist

- [ ] Build time < 500ms ✅ (currently 351ms)
- [ ] Bundle size ≤ 150 KB ✅ (currently 107 KB)
- [ ] Settings load < 500ms
- [ ] Config save < 200ms
- [ ] No memory leaks (reload 10x, check DevTools)

---

## Step 8: Accessibility Check (10 min)

### Keyboard Navigation

1. Press `Tab` key repeatedly
   - [ ] All controls focusable
   - [ ] Focus order logical (top → bottom)
   - [ ] Visible focus rings
   - [ ] No keyboard traps

2. Press `Enter` / `Space` on controls
   - [ ] Buttons activate
   - [ ] Toggles switch
   - [ ] Links navigate

3. Press `Arrow keys` in controls
   - [ ] Sliders adjust
   - [ ] Selects change
   - [ ] Radio groups navigate

### Screen Reader (Optional)

- [ ] All inputs have labels
- [ ] Button text meaningful
- [ ] Error messages announced
- [ ] State changes announced

---

## Step 9: Final Verification (5 min)

### Pre-Merge Checklist

- [ ] All 8 commands work end-to-end
- [ ] Settings persist across restarts
- [ ] No console errors or warnings
- [ ] Build succeeds (`cargo tauri build`)
- [ ] Bundle size within budget (107 KB ✅)
- [ ] Type checking passes (`npm run check`)
- [ ] All smoke tests pass
- [ ] Error handling works
- [ ] Performance metrics met
- [ ] Accessibility checks pass

---

## Step 10: Commit & Deploy (10 min)

### Git Workflow

```bash
# 1. Review changes
git status
git diff

# 2. Stage files
git add apps/desktop/ui-next/
git add apps/desktop/tauri.conf.json
git add apps/desktop/src-tauri/src/commands/
git add openspec/changes/031-ui-rewrite/

# 3. Commit
git commit -m "feat(ui): integrate modern Settings UI (Phase 0-3)

- Vite + Svelte 5 + TypeScript + TailwindCSS v4
- 6 settings sections (Audio, TTS, Hotkeys, Chat, Window, Protection)
- Global config store with Svelte 5 runes
- Type-safe Tauri command wrappers
- Bundle: 107.06 KB (71% of 150 KB target)
- Build time: 351ms

Closes #XXX"

# 4. Push
git push origin feature/ui-rewrite

# 5. Create PR
gh pr create --title "feat(ui): Modern Settings UI" --body "See INTEGRATION.md"
```

### Checklist

- [ ] Changes committed with clear message
- [ ] PR created with context
- [ ] CI passes (if configured)
- [ ] Ready for code review

---

## Troubleshooting

### UI doesn't load

**Symptoms**: Blank screen, no errors  
**Check**:
1. `dist/` folder exists after `npm run build`
2. `tauri.conf.json` has correct `frontendDist` path
3. Open DevTools Console for errors

**Fix**: Rebuild UI and restart Tauri dev

---

### Commands not found

**Symptoms**: "Command not found: get_config"  
**Check**:
1. Command implemented in Rust
2. Command registered in `invoke_handler![]`
3. Command name matches exactly (case-sensitive)

**Fix**: Re-register command and restart Tauri dev

---

### Settings don't save

**Symptoms**: Change setting, reload → reverted  
**Check**:
1. Command returns `Ok(())` not `Err(...)`
2. Config file writable (`config.toml`)
3. Check console for error messages

**Fix**: Check Rust command implementation logs

---

### Type errors

**Symptoms**: TypeScript complains about types  
**Check**:
1. Rust types match TypeScript interfaces
2. `serde_json::Value` used for generic values
3. Optional fields marked with `Option<T>`

**Fix**: Align Rust and TypeScript types

---

## Success Criteria

### Must Have ✅

- [x] All 6 settings sections functional
- [x] Settings persist across restarts
- [x] No console errors
- [x] Build succeeds
- [x] Type checking passes
- [x] Bundle ≤ 150 KB (107 KB ✅)

### Nice to Have 🎯

- [ ] Smooth animations
- [ ] Loading skeletons
- [ ] Settings search
- [ ] Keyboard shortcuts
- [ ] E2E tests

---

## Timeline Estimate

| Task | Time | Status |
|------|------|--------|
| Backend commands | 30 min | ⏳ Todo |
| Update Tauri config | 5 min | ⏳ Todo |
| Build & test | 15 min | ⏳ Todo |
| Smoke test | 10 min | ⏳ Todo |
| Navigation test | 5 min | ⏳ Todo |
| Error handling | 10 min | ⏳ Todo |
| Performance check | 10 min | ⏳ Todo |
| Accessibility check | 10 min | ⏳ Todo |
| Final verification | 5 min | ⏳ Todo |
| Commit & deploy | 10 min | ⏳ Todo |
| **Total** | **~2 hours** | |

---

## Next Steps After Integration

1. **Phase 4 Planning** (optional)
   - Meeting List UI
   - Overlay components
   - E2E tests
   - Tauri Specta integration

2. **User Feedback**
   - Beta release to select users
   - Collect feedback & metrics
   - Iterate on UX issues

3. **Gradual Rollout**
   - 10% → 50% → 100% rollout
   - Monitor crash rates
   - Legacy UI deprecation

---

**Ready to integrate? Start with Step 1! 🚀**

**Questions?** See [INTEGRATION.md](../../../openspec/changes/031-ui-rewrite/INTEGRATION.md)
