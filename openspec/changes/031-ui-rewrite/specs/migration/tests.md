# Test Specifications: Migration

## Integration Tests

### Parallel UI Deployment

**Test: dual_ui_accessible**
```bash
# Build both UIs
cd ui && npm run build  # Old UI (if still using npm)
cd ../ui-next && npm run build
cargo build -p desktop

# Start app
./target/debug/desktop.exe

# Manual test:
# 1. Press Ctrl+O → old UI window opens (overlay.html)
# 2. Press Ctrl+Shift+O → new UI window opens (index.html from ui-next/dist)
# 3. Both windows functional, share Rust backend state
# 4. Model selection in one → pill updates in both
```

**Test: config_default_ui_flag**
```toml
# config.toml
[ui]
default_version = "next"
```
```bash
cargo run -p desktop
# Press Ctrl+O → new UI opens
# Press Ctrl+Alt+O → old UI opens (swap)
```

---

### Feature Parity Validation

**Test: feature_parity_checklist_complete**
```markdown
# openspec/changes/031-ui-rewrite/feature-parity-checklist.md

- [x] Model selection modal (with search, families, metadata)
- [x] Chat render (markdown, code highlight, streaming)
- [x] Context slider modal
- [x] Hotkeys editor modal
- [x] TTS controls (on/off/mode toggle)
- [x] Opacity slider (live preview)
- [x] Accent color picker
- [x] Screenshot (full/region)
- [x] Status indicator (corner position, rail mode)
- [x] Click-through mode toggle
- [x] Mute toggle
- [x] Reasoning effort slider
- [x] Chat collapse (transcripts, operations, last message)
- [x] Autoscroll behavior
- [x] Config persistence (all settings save to config.toml)
```

**Test: e2e_feature_parity_suite**
```ts
// ui-next/tests/e2e/feature-parity.spec.ts
import { test } from '@playwright/test';

const features = [
  { name: 'Model selection', test: async ({ page }) => { /* ... */ } },
  { name: 'Chat streaming', test: async ({ page }) => { /* ... */ } },
  { name: 'Context settings', test: async ({ page }) => { /* ... */ } },
  // ... all 15 features
];

features.forEach(({ name, test: testFn }) => {
  test(`feature parity: ${name}`, testFn);
});

test.describe('Feature Parity Suite', () => {
  test('all features pass', async ({ page }) => {
    // Summary test: run all features, ensure 100% pass rate
  });
});
```

**Test: manual_dogfooding_1_week**
```markdown
# Dogfooding Log (track in GitHub issue)

Day 1 (2024-01-08):
- [ ] Opened modal, selected 5 models → all worked
- [ ] Sent 20 messages, chat streamed correctly
- [ ] Changed context settings → saved to config
- [x] Bug found: tooltip flickers on hover (filed #123, P2)

Day 2-7:
- [ ] ... (continue daily usage)

Summary:
- P0 bugs: 0
- P1 bugs: 2 (backlog, not blockers)
- P2 bugs: 5 (polish items)
→ Cutover approved ✅
```

---

### Rollback Plan

**Test: rollback_via_config**
```bash
# Simulate critical bug after cutover
echo "Bug: chat not rendering"

# Rollback step 1: edit config
cat >> config.toml << EOF
[ui]
default_version = "legacy"
EOF

# Restart app
cargo run -p desktop --release

# Verify: old UI loads, chat works
# Expected: ui-legacy/ served, overlay.html renders correctly
```

**Test: rollback_via_binary**
```bash
# Build hotfix with legacy flag
cargo build -p desktop --release --features use-legacy-ui

# Distribute to users
./target/release/desktop.exe --use-legacy-ui

# Verify: old UI loads without config change
```

**Test: rollback_via_git**
```bash
# Emergency rollback
git log --oneline -5
# abc123 ui: cutover to Svelte 5 (change 031)
# def456 ...

git revert abc123
cargo build -p desktop --release

# Verify: old UI restored, ui-next/ ignored
```

---

### Cutover Execution

**Test: cutover_atomic**
```bash
#!/bin/bash
# scripts/cutover.sh

set -e  # Exit on error

echo "Starting cutover..."

# Step 1: Backup
cp -r ui ui-backup-$(date +%s)

# Step 2: Rename
mv ui ui-legacy
mv ui-next/dist ui

# Step 3: Update tauri config
sed -i 's|"frontendDist": "./ui-next/dist"|"frontendDist": "./ui"|' apps/desktop/tauri.conf.json

# Step 4: Verify build
npm --prefix ui-next run build
cargo build -p desktop --release

# Step 5: Smoke test
timeout 30s ./target/release/desktop.exe &
DESKTOP_PID=$!
sleep 5

if ps -p $DESKTOP_PID > /dev/null; then
  echo "✓ Smoke test passed"
  kill $DESKTOP_PID
else
  echo "✗ Smoke test failed"
  exit 1
fi

# Step 6: Git commit
git add -A
git commit -m "ui: cutover to Svelte 5 (change 031)"

echo "Cutover complete. Grace period: 2 weeks."
```

**Test: cutover_smoke_test**
```ts
// Automated smoke test after cutover
test('cutover smoke test', async ({ page }) => {
  await page.goto('http://localhost:1430');
  
  // Critical paths must work
  await page.click('[data-testid="model-pill"]');
  await expect(page.locator('[role="dialog"]')).toBeVisible();
  
  await page.click('text=Anthropic');
  await page.click('text=claude-sonnet-5');
  
  await expect(page.locator('[data-testid="model-pill"]')).toContainText('Claude');
  
  // If any critical path fails → rollback immediately
});
```

---

### Deprecation Timeline

**Test: grace_period_2_weeks**
```bash
# Day 1 (cutover): ui-legacy/ exists
ls ui-legacy/
# Expected: overlay.html, overlay.js, overlay.css

# Day 14 (end of grace period): check for P0 bugs
gh issue list --label "ui-next,P0" --state open
# Expected: 0 open P0 bugs

# If 0 P0 bugs → proceed to deprecation
```

**Test: legacy_ui_removed**
```bash
# After 2-week grace period
git tag ui-legacy-final HEAD
git rm -rf ui-legacy/
git commit -m "ui: remove legacy UI after 2-week grace period"
git push origin main --tags

# Verify: ui-legacy/ deleted, tag preserves history
git show ui-legacy-final:ui-legacy/overlay.html
# Expected: shows file content from tag
```

---

## Performance Regression Detection

**Test: ci_bundle_size_check**
```yaml
# .github/workflows/ui-checks.yml
name: UI Checks

on: [pull_request]

jobs:
  bundle-size:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # Fetch all history for comparison
      
      - name: Build PR branch
        run: |
          cd ui-next
          npm ci
          npm run build
          SIZE=$(du -sb dist | cut -f1)
          echo "pr_size=$SIZE" >> $GITHUB_OUTPUT
        id: pr
      
      - name: Build base branch
        run: |
          git checkout ${{ github.base_ref }}
          cd ui-next
          npm ci
          npm run build
          SIZE=$(du -sb dist | cut -f1)
          echo "base_size=$SIZE" >> $GITHUB_OUTPUT
        id: base
      
      - name: Compare sizes
        run: |
          PR=${{ steps.pr.outputs.pr_size }}
          BASE=${{ steps.base.outputs.base_size }}
          DIFF=$((PR - BASE))
          PCT=$((DIFF * 100 / BASE))
          
          echo "Base: ${BASE} bytes"
          echo "PR: ${PR} bytes"
          echo "Diff: ${DIFF} bytes (${PCT}%)"
          
          if [ $PCT -gt 5 ]; then
            echo "::error::Bundle grew ${PCT}% (exceeds 5% threshold)"
            exit 1
          fi
```

**Test: ci_lighthouse_check**
```yaml
      - name: Lighthouse CI
        uses: treosh/lighthouse-ci-action@v9
        with:
          urls: |
            http://localhost:1430
          configPath: ./lighthouserc.json
          uploadArtifacts: true
```

```json
// lighthouserc.json
{
  "ci": {
    "assert": {
      "assertions": {
        "categories:performance": ["error", {"minScore": 0.9}],
        "categories:accessibility": ["error", {"minScore": 0.95}],
        "first-contentful-paint": ["warn", {"maxNumericValue": 2000}],
        "interactive": ["error", {"maxNumericValue": 5000}]
      }
    }
  }
}
```

---

## Documentation Validation

**Test: readme_reflects_new_stack**
```bash
# Verify README.md updated
grep -q "npm run dev" README.md
grep -q "Svelte 5" README.md
grep -q "TypeScript" README.md

# Verify links to docs
grep -q "docs/ui/SETUP.md" README.md
grep -q "docs/ui/COMPONENTS.md" README.md
```

**Test: contributing_has_new_workflow**
```bash
# Verify CONTRIBUTING.md updated
grep -q "npm run lint" CONTRIBUTING.md
grep -q "npm run type-check" CONTRIBUTING.md
grep -q "npm test" CONTRIBUTING.md
grep -q "Vitest" CONTRIBUTING.md
grep -q "Playwright" CONTRIBUTING.md
```

---

## Acceptance Criteria

**Migration complete when:**
- [ ] Parallel run: both UIs work simultaneously for 1 week
- [ ] Feature parity: 15/15 features validated (manual + E2E)
- [ ] Dogfooding: 1 week usage, 0 P0 bugs
- [ ] Performance: no regressions (bundle size, render time, memory)
- [ ] Accessibility: Lighthouse ≥95 maintained
- [ ] Cutover: atomic script executes successfully, smoke test passes
- [ ] Grace period: 2 weeks, 0 P0 bugs → legacy UI deleted
- [ ] Documentation: README, CONTRIBUTING, docs/ui/* updated
- [ ] CI: bundle size check, Lighthouse CI green
- [ ] Rollback plan: tested (config, binary, git revert methods work)

**Post-cutover monitoring (3 months):**
- [ ] Zero critical regressions filed
- [ ] Developer velocity: new features ship 2x faster (PR cycle time)
- [ ] Bundle size stable (no >10% growth)
- [ ] User feedback: NPS/satisfaction survey shows ≥85% approval
