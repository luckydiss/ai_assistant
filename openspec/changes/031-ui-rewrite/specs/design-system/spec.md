# Delta: Design System (ADDED)

## ADDED Requirements

### Requirement: Design Tokens (CSS Variables)
Система SHALL определять semantic tokens для colors, spacing, typography, borders, shadows в `:root`.

#### Scenario: Все токены определены (test: tokens_defined)
- GIVEN файл `ui-next/src/lib/design/tokens.css`
- WHEN парсится как CSS
- THEN содержит переменные:
  - Colors: `--color-bg-primary`, `--color-text-primary`, `--color-accent`, `--color-error`
  - Spacing: `--space-{1,2,3,4,6,8}` (4px base scale)
  - Typography: `--text-{xs,sm,base,lg}`, `--font-sans`, `--font-mono`
  - Borders: `--radius-{sm,md,lg}`, `--color-border`
  - Shadows: `--shadow-{sm,md,lg}`

#### Scenario: Tailwind extends tokens (test: tailwind_uses_tokens)
- GIVEN `tailwind.config.ts` с `extend.colors.bg.primary: 'var(--color-bg-primary)'`
- WHEN компилируется `class="bg-bg-primary"`
- THEN CSS содержит `background-color: var(--color-bg-primary)`

### Requirement: Primitive Components Library
Система SHALL предоставлять 8+ переиспользуемых UI примитивов.

#### Scenario: Button компонент (test: button_variants)
- GIVEN `<Button variant="primary">Save</Button>`
- WHEN рендерится
- THEN кнопка имеет `bg-accent`, `text-white`, hover state, focus ring
- AND поддерживает variants: `primary | secondary | ghost | danger`
- AND поддерживает sizes: `sm | md | lg`
- AND `disabled` и `loading` props корректно отображаются

#### Scenario: Modal компонент (test: modal_a11y)
- GIVEN `<Modal open={true} title="Test">Content</Modal>`
- WHEN рендерится
- THEN содержит `role="dialog"`, `aria-labelledby` указывает на title
- AND фокус trapped внутри модалки (Tab циклически)
- AND Escape закрывает модалку
- AND backdrop click закрывает (если `closeOnBackdrop={true}`)

#### Scenario: Input компонент (test: input_controlled)
- GIVEN `<Input bind:value={text} placeholder="Name" />`
- WHEN пользователь вводит "test"
- THEN `text` обновляется реактивно на каждый keystroke
- AND placeholder скрыт, label анимируется вверх (если есть)

#### Scenario: Slider компонент (test: slider_range)
- GIVEN `<Slider bind:value={val} min={0} max={100} step={5} />`
- WHEN пользователь перемещает handle
- THEN `val` обновляется кратно 5, визуально отображается track fill
- AND поддерживает keyboard (Arrow keys)

### Requirement: Accessibility Guidelines
Все примитивы SHALL соответствовать WCAG 2.1 Level AA.

#### Scenario: Contrast ratios (test: color_contrast_aa)
- GIVEN все пары text/background в токенах
- WHEN проверяются через axe-core
- THEN все проходят минимум 4.5:1 для нормального текста, 3:1 для крупного

#### Scenario: Keyboard navigation (test: keyboard_nav_modal)
- GIVEN открытая модалка с формой (input + 2 buttons)
- WHEN пользователь нажимает Tab
- THEN фокус циклится: input → button1 → button2 → input
- AND Shift+Tab идёт в обратном порядке

#### Scenario: Screen reader announcements (test: toast_aria_live)
- GIVEN `<Toast message="Saved" />`
- WHEN toast появляется
- THEN имеет `role="status"` и `aria-live="polite"`
- AND NVDA/VoiceOver объявляют "Saved"

### Requirement: Icon Library Integration
Система SHALL использовать tree-shakeable icons (Lucide).

#### Scenario: Только используемые иконки в bundle (test: icons_tree_shaken)
- GIVEN импортируются только `Search`, `X`, `Check` из lucide-svelte
- WHEN `npm run build`
- THEN bundle НЕ содержит неиспользованные иконки (например, `Calendar`)
- AND размер lucide chunk ≤ 5kb gzipped

### Requirement: Dark Mode Support
Система SHALL поддерживать dark/light/auto темы через data-attribute.

#### Scenario: Dark mode (test: theme_dark)
- GIVEN `document.documentElement.dataset.theme = 'dark'`
- WHEN компонент рендерится
- THEN использует dark tokens (`--color-bg-primary: #0a0a0a`)

#### Scenario: Auto theme (test: theme_auto_prefers_dark)
- GIVEN `dataset.theme = 'auto'` И `prefers-color-scheme: dark`
- WHEN компонент рендерится
- THEN применяется dark mode

## MODIFIED Requirements
(нет)

## REMOVED Requirements
(нет)
