# Delta: Overlay UI

## ADDED Requirements

### Requirement: Transparent Always-On-Top Window
Система SHALL показывать карточку ответов в прозрачном окне поверх всех приложений.

#### Scenario: Поверх Zoom (test: manual_above_zoom)
- GIVEN открыт Zoom во весь экран
- WHEN приложение запущено
- THEN карточка видна поверх Zoom (manual)

#### Scenario: Не мешает кликам (test: manual_click_through)
- GIVEN оверлей открыт
- WHEN клик вне карточки
- THEN клик проходит в приложение под оверлеем (manual)

### Requirement: Streaming Render
Система SHALL дописывать ответ по мере прихода answer_token событий.

#### Scenario: Инкрементальный рендер (test: manual_stream_render)
- GIVEN идёт стрим ответа
- WHEN приходят answer_token
- THEN текст появляется постепенно, без полной перерисовки мигания (manual)

### Requirement: Markdown Subset
Система SHALL рендерить code-блоки, буллеты и bold.

#### Scenario: Code block (test: manual_code_block)
- GIVEN ответ содержит ```-блок
- WHEN отрендерен
- THEN код в <pre> с моноширинным шрифтом (manual)

### Requirement: Status Indicator
Система SHALL показывать текущий статус: listening / generating / error.

#### Scenario: Статус обновляется (test: manual_status)
- WHEN приходят status-события
- THEN индикатор меняет цвет/текст (manual)

## MODIFIED Requirements
(none)

## REMOVED Requirements
(none)
