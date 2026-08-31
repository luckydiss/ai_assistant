# Test Specifications: Foundation

## Unit Tests

### TypeScript Compilation

**Test: tsc_compiles_without_errors**
```bash
cd ui-next
npx tsc --noEmit
# Expected: exit code 0, no output
```

**Test: tsc_fails_on_type_error**
```ts
// ui-next/src/__tests__/type-errors.test.ts
import { describe, it, expect } from 'vitest';
import { exec } from 'child_process';

it('should fail compilation on type mismatch', async () => {
  // Create temp file with type error
  const code = `
    import { commands } from '$lib/bindings';
    const models: string = await commands.modelsList(); // error: string not assignable to ModelMetadata[]
  `;
  // Write to temp file, run tsc, expect exit !== 0
});
```

### Tauri Specta

**Test: specta_generates_types**
```rust
// apps/desktop/tests/specta.rs
#[test]
fn bindings_generated() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/ui-next/src/lib/bindings.ts");
    assert!(std::path::Path::new(path).exists());
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("export type ModelMetadata"));
    assert!(content.contains("export const commands"));
}
```

**Test: specta_updates_on_change**
```bash
# Manual test:
# 1. Change models_list return type in commands.rs
# 2. cargo build -p desktop
# 3. Check bindings.ts updated
# 4. TypeScript code with old type fails tsc
```

### Vite Dev Server

**Test: dev_server_invokes_rust** (manual)
```bash
# Terminal 1:
cd ui-next && npm run dev
# Terminal 2:
cargo run -p desktop
# Browser: http://localhost:5173
# Open DevTools Console, run:
# await commands.modelsList()
# Expected: returns array of models from Rust
```

### Build Integration

**Test: tauri_serves_new_ui**
```bash
cd ui-next && npm run build
cargo build -p desktop --release
./target/release/desktop.exe
# Expected: overlay loads ui-next/dist/index.html
# Verify: DevTools Sources tab shows compiled Svelte chunks
```

**Test: tauri_fallback_old_ui**
```bash
rm -rf ui-next/dist
cargo build -p desktop
# Expected: tauri.conf.json uses frontendDist="./ui" (old)
```

### Bundle Size

**Test: bundle_size_within_budget**
```js
// scripts/check-bundle-size.js
import { statSync } from 'fs';
import { globSync } from 'glob';
import { gzipSync } from 'zlib';

const files = globSync('ui-next/dist/**/*.js');
let total = 0;
files.forEach(f => {
  const content = readFileSync(f);
  total += gzipSync(content, { level: 9 }).length;
});

const BUDGET = 80 * 1024; // 80kb
if (total > BUDGET) {
  console.error(`Bundle ${(total/1024).toFixed(1)}kb exceeds budget 80kb`);
  process.exit(1);
}
console.log(`✓ Bundle ${(total/1024).toFixed(1)}kb within budget`);
```

**Run in CI**:
```yaml
# .github/workflows/ui.yml
- name: Check bundle size
  run: npm run build && node scripts/check-bundle-size.js
```

---

## Integration Tests

### ESLint Pre-commit Hook

**Test: eslint_blocks_commit**
```bash
# Setup: husky + lint-staged
npx husky install
npx husky add .husky/pre-commit "npx lint-staged"
# .lintstagedrc.json:
# { "*.{ts,svelte}": "eslint --max-warnings 0" }

# Test:
echo "let unused = 1;" >> ui-next/src/temp.ts
git add ui-next/src/temp.ts
git commit -m "test"
# Expected: commit rejected, stderr contains "unused is assigned but never used"
```

---

## Performance Tests

### Bundle Size Regression

**Test: bundle_size_regression_ci**
```yaml
# .github/workflows/ui.yml
- name: Build and measure
  run: |
    npm run build
    SIZE=$(node scripts/check-bundle-size.js --json | jq .total)
    echo "bundle_size=$SIZE" >> $GITHUB_OUTPUT
    
- name: Compare to base
  run: |
    BASE_SIZE=$(gh api repos/$REPO/commits/$BASE_SHA/statuses | jq '.[] | select(.context=="bundle-size") | .description | tonumber')
    DIFF=$(( $SIZE - $BASE_SIZE ))
    PCT=$(( $DIFF * 100 / $BASE_SIZE ))
    if [ $PCT -gt 5 ]; then
      echo "Bundle grew $PCT% (${DIFF}kb)"
      exit 1
    fi
```

---

## Acceptance Criteria

**Foundation complete when:**
- [ ] `npm run build` exits 0, generates `dist/` with assets
- [ ] `npx tsc --noEmit` exits 0 (no type errors)
- [ ] `npm run lint` exits 0 (ESLint + Prettier pass)
- [ ] `bindings.ts` contains all 40+ commands with correct types
- [ ] Bundle size ≤80kb gzipped
- [ ] Pre-commit hook blocks commits with lint errors
- [ ] Tauri serves `ui-next/dist/` when exists, fallback to `ui/`
