# Delta: UI (overlay v2)

## MODIFIED Requirements

### Requirement: Transparent Always-On-Top Window
Система SHALL иметь два окна: main (панель управления, обычное) и overlay (прозрачный, always-on-top, skipTaskbar, stealth-affinity).

#### Scenario: Два окна (test: manual_two_windows)
- GIVEN приложение запущено
- WHEN открыты оба окна
- THEN overlay поверх Zoom, main — обычное окно (manual)

## ADDED Requirements

### Requirement: Overlay Chat
Оверлей SHALL показывать ленту: реплики I/C и стриминговый ответ с markdown.

#### Scenario: Лента диалога (test: manual_chat_feed)
- WHEN приходят turn и answer_token
- THEN реплики и ответ видны в ленте, автопрокрутка вниз (manual)

### Requirement: Quick Actions
Оверлей SHALL иметь кнопки «Что сказать» (manual_trigger) и «Резюме» (manual с note "сжато перескажи суть диалога").

#### Scenario: Что сказать (test: manual_what_to_say)
- WHEN нажата кнопка
- THEN приходит ответ по последнему вопросу (manual)

### Requirement: Manual Input
Оверлей SHALL иметь поле ввода; отправка = manual_trigger(note).

#### Scenario: Ручной вопрос (test: manual_input)
- WHEN введён текст и отправлен
- THEN ответ учитывает его (manual)

### Requirement: Mic Mute
Оверлей SHALL переключать захват микрофона; в mute реплики C не поступают.

#### Scenario: Mute (test: manual_mute)
- WHEN mute включён и пользователь говорит
- THEN реплики C не появляются в ленте (manual)

### Requirement: VAD State Indicator
Оверлей SHALL показывать текущую стадию схемы: ожидание/запись/пауза/отправка.

#### Scenario: Стадии видны (test: manual_vad_states)
- WHEN идёт речь
- THEN индикатор переходит ожидание→запись→пауза→отправка (manual)

### Requirement: Protection and Model Badges
Оверлей SHALL показывать бейдж защиты (affinity применён/нет) и модель из конфига.

#### Scenario: Бейджи (test: manual_badges)
- WHEN приложение запущено
- THEN видны "Защита вкл." и имя модели (manual)

### Requirement: Click-Through Toggle
Оверлей SHALL переключать кликабельность (set_ignore_cursor_events) по хоткею Ctrl+W.

#### Scenario: Click-through (test: manual_click_through_toggle)
- WHEN нажат Ctrl+W
- THEN клики проходят сквозь окно; повторное нажатие возвращает кликабельность (manual)

### Requirement: Meetings View
Main-окно SHALL показывать список встреч (группировка по датам, поиск), создание (имя+вакансия), continue, удаление.

#### Scenario: CRUD встреч в UI (test: manual_meetings_view)
- WHEN создана встреча и нажато «Продолжить»
- THEN встреча в списке, пайплайн стартует, overlay активен (manual)

### Requirement: Contexts View
Main-окно SHALL показывать редактор контекста: name, role, languages (до 3), resume (textarea/файл), extra_prompt; сохранение; назначение на встречу.

#### Scenario: Редактор контекста (test: manual_contexts_view)
- WHEN контекст сохранён и назначен встрече
- THEN ответы учитывают role/extra (manual)

## REMOVED Requirements
(none)
