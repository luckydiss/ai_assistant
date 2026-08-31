# Test Specifications: Design System

## Unit Tests

### Tokens

**Test: tokens_defined**
```ts
// ui-next/src/lib/design/__tests__/tokens.test.ts
import { describe, it, expect } from 'vitest';

it('should define all color tokens', () => {
  const root = getComputedStyle(document.documentElement);
  expect(root.getPropertyValue('--color-bg-primary')).toBeTruthy();
  expect(root.getPropertyValue('--color-accent')).toBeTruthy();
  expect(root.getPropertyValue('--color-error')).toBeTruthy();
});

it('should define spacing scale', () => {
  const root = getComputedStyle(document.documentElement);
  [1,2,3,4,6,8].forEach(n => {
    expect(root.getPropertyValue(`--space-${n}`)).toBeTruthy();
  });
});
```

**Test: tailwind_uses_tokens**
```ts
import { render } from '@testing-library/svelte';

it('should map Tailwind classes to CSS variables', () => {
  const { container } = render('<div class="bg-bg-primary"></div>');
  const div = container.querySelector('div');
  const bg = getComputedStyle(div!).backgroundColor;
  // bg should resolve to var(--color-bg-primary)
  expect(bg).toBe(getComputedStyle(document.documentElement).getPropertyValue('--color-bg-primary'));
});
```

---

### Button Component

**Test: button_variants**
```ts
// ui-next/src/lib/design/components/__tests__/Button.test.ts
import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import Button from '../Button.svelte';

describe('Button', () => {
  it('renders primary variant', () => {
    const { container } = render(Button, { props: { variant: 'primary', children: 'Save' } });
    const btn = container.querySelector('button');
    expect(btn).toHaveClass('bg-accent');
  });

  it('renders all sizes', () => {
    ['sm', 'md', 'lg'].forEach(size => {
      const { container } = render(Button, { props: { size, children: 'Test' } });
      const btn = container.querySelector('button');
      expect(btn).toHaveClass(size === 'sm' ? 'h-8' : size === 'md' ? 'h-10' : 'h-12');
    });
  });

  it('disables when loading', () => {
    const { container } = render(Button, { props: { loading: true, children: 'Loading' } });
    const btn = container.querySelector('button') as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
    expect(container.querySelector('.animate-spin')).toBeTruthy(); // Loader2 icon
  });

  it('calls onclick handler', async () => {
    const onclick = vi.fn();
    const { getByRole } = render(Button, { props: { onclick, children: 'Click' } });
    await fireEvent.click(getByRole('button'));
    expect(onclick).toHaveBeenCalledOnce();
  });

  it('does not call onclick when disabled', async () => {
    const onclick = vi.fn();
    const { getByRole } = render(Button, { props: { disabled: true, onclick, children: 'Test' } });
    await fireEvent.click(getByRole('button'));
    expect(onclick).not.toHaveBeenCalled();
  });
});
```

---

### Modal Component

**Test: modal_a11y**
```ts
// ui-next/src/lib/design/components/__tests__/Modal.test.ts
import { render, fireEvent } from '@testing-library/svelte';
import { axe } from '@axe-core/playwright';
import Modal from '../Modal.svelte';

describe('Modal', () => {
  it('has correct ARIA attributes', async () => {
    const { container } = render(Modal, { props: { open: true, title: 'Test Modal' } });
    const dialog = container.querySelector('[role="dialog"]');
    expect(dialog).toBeTruthy();
    expect(dialog?.getAttribute('aria-labelledby')).toBeTruthy();
    
    // axe-core accessibility check
    const results = await axe(container);
    expect(results.violations).toHaveLength(0);
  });

  it('traps focus with Tab', async () => {
    const { container } = render(Modal, {
      props: {
        open: true,
        children: '<input id="a"/><input id="b"/><button id="c">Close</button>',
      },
    });
    
    const a = container.querySelector('#a') as HTMLElement;
    const b = container.querySelector('#b') as HTMLElement;
    const c = container.querySelector('#c') as HTMLElement;
    
    a.focus();
    await fireEvent.keyDown(a, { key: 'Tab' });
    expect(document.activeElement).toBe(b);
    
    await fireEvent.keyDown(b, { key: 'Tab' });
    expect(document.activeElement).toBe(c);
    
    await fireEvent.keyDown(c, { key: 'Tab' });
    expect(document.activeElement).toBe(a); // wraps to first
  });

  it('closes on Escape', async () => {
    let open = $state(true);
    const { container } = render(Modal, { props: { open, title: 'Test' } });
    await fireEvent.keyDown(container, { key: 'Escape' });
    expect(open).toBe(false);
  });

  it('closes on backdrop click', async () => {
    const onclose = vi.fn();
    const { container } = render(Modal, { props: { open: true, closeOnBackdrop: true, onclose } });
    const backdrop = container.querySelector('.modal-backdrop');
    await fireEvent.click(backdrop!);
    expect(onclose).toHaveBeenCalled();
  });
});
```

---

### Input Component

**Test: input_controlled**
```ts
// ui-next/src/lib/design/components/__tests__/Input.test.ts
import { render, fireEvent } from '@testing-library/svelte';
import Input from '../Input.svelte';

it('updates value on input', async () => {
  let value = $state('');
  const { getByRole } = render(Input, { props: { value, placeholder: 'Name' } });
  const input = getByRole('textbox') as HTMLInputElement;
  
  await fireEvent.input(input, { target: { value: 'test' } });
  expect(value).toBe('test');
});

it('shows error state', () => {
  const { container } = render(Input, { props: { error: 'Required field' } });
  const input = container.querySelector('input');
  expect(input).toHaveClass('border-error');
  expect(container.textContent).toContain('Required field');
});
```

---

### Slider Component

**Test: slider_range**
```ts
// ui-next/src/lib/design/components/__tests__/Slider.test.ts
import { render, fireEvent } from '@testing-library/svelte';
import Slider from '../Slider.svelte';

it('updates value within min/max', async () => {
  let value = $state(50);
  const { getByRole } = render(Slider, { props: { value, min: 0, max: 100, step: 5 } });
  const slider = getByRole('slider') as HTMLInputElement;
  
  await fireEvent.input(slider, { target: { value: '75' } });
  expect(value).toBe(75);
});

it('supports keyboard arrows', async () => {
  let value = $state(10);
  const { getByRole } = render(Slider, { props: { value, min: 0, max: 100, step: 5 } });
  const slider = getByRole('slider');
  
  await fireEvent.keyDown(slider, { key: 'ArrowUp' });
  expect(value).toBe(15);
  
  await fireEvent.keyDown(slider, { key: 'ArrowDown' });
  expect(value).toBe(10);
});
```

---

## Visual Regression Tests

### Dark Mode

**Test: theme_dark** (Playwright visual)
```ts
// ui-next/tests/visual/theme.spec.ts
import { test, expect } from '@playwright/test';

test('dark mode applies correct colors', async ({ page }) => {
  await page.goto('http://localhost:5173');
  await page.evaluate(() => {
    document.documentElement.dataset.theme = 'dark';
  });
  
  const bg = await page.evaluate(() => {
    return getComputedStyle(document.documentElement).getPropertyValue('--color-bg-primary');
  });
  expect(bg).toBe('#0a0a0a');
  
  // Screenshot comparison
  await expect(page).toHaveScreenshot('dark-mode.png');
});

test('auto theme respects prefers-color-scheme', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'dark' });
  await page.goto('http://localhost:5173');
  await page.evaluate(() => {
    document.documentElement.dataset.theme = 'auto';
  });
  
  const bg = await page.evaluate(() => {
    return getComputedStyle(document.documentElement).getPropertyValue('--color-bg-primary');
  });
  expect(bg).toBe('#0a0a0a');
});
```

---

## Accessibility Tests

### Color Contrast

**Test: color_contrast_aa**
```ts
// ui-next/tests/a11y/contrast.spec.ts
import { test, expect } from '@playwright/test';
import { injectAxe, checkA11y } from '@axe-core/playwright';

test('all text meets WCAG AA contrast', async ({ page }) => {
  await page.goto('http://localhost:5173');
  await injectAxe(page);
  
  await checkA11y(page, null, {
    rules: {
      'color-contrast': { enabled: true },
    },
  });
});
```

### Screen Reader

**Test: toast_aria_live** (manual)
```
1. Enable NVDA (Windows) or VoiceOver (macOS)
2. Open application
3. Trigger toast: click "Save" button
4. Expected: NVDA announces "Saved successfully" without interrupting
5. Toast has role="status" and aria-live="polite"
```

---

## Integration Tests

### Icon Tree-Shaking

**Test: icons_tree_shaken**
```bash
# Build and check bundle
npm run build
npx vite-bundle-visualizer --json > bundle-stats.json

# Check lucide chunk size
node -e "
const stats = require('./bundle-stats.json');
const lucideChunk = stats.find(c => c.label.includes('lucide'));
const gzipSize = lucideChunk.gzipLength;
if (gzipSize > 5 * 1024) {
  console.error('Lucide chunk', gzipSize, 'exceeds 5kb');
  process.exit(1);
}
console.log('✓ Lucide chunk', gzipSize, 'bytes');
"
```

---

## Acceptance Criteria

**Design System complete when:**
- [ ] All 8 primitives (Button, Modal, Input, Slider, Select, Checkbox, Badge, Toast) implemented
- [ ] Unit tests for each primitive: 100% coverage
- [ ] axe-core: 0 violations for all components
- [ ] WCAG AA contrast: all text/background pairs pass
- [ ] Keyboard navigation: Tab, Arrow keys, Enter, Escape work in all components
- [ ] Visual regression: dark mode screenshot matches baseline
- [ ] Bundle: Lucide chunk ≤5kb gzip
- [ ] Storybook/docs (optional): each primitive documented with examples
