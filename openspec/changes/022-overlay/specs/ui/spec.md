# Delta: UI (overlay 1:1)

## MODIFIED Requirements

### Requirement: Overlay Layout
Оверлей SHALL иметь зоны: топбар, левый rail чатов, лента, quick-actions, инпут, нижняя тулбар; поверхность ленты полупрозрачна (просвечивание — фича), прозрачность = [ui] opacity, акцент = [ui] accent (CSS-переменные).

#### Scenario: Прозрачность из конфига (test: manual_opacity_accent)
- GIVEN [ui] opacity=70, accent=blue
- WHEN оверлей открыт
- THEN поверхность ~70% непрозрачна, кнопки/активные состояния синие (manual)

### Requirement: Chats Rail
Rail SHALL показывать номера чатов встречи, кнопку «+»; клик переключает активный чат (своя история, свой контекст); тумблер «скрыть чаты» скрывает rail.

#### Scenario: Переключение чата (test: manual_chat_switch)
- GIVEN два чата с разными историями
- WHEN клик по номеру 2
- THEN лента показывает историю чата 2, новые turn пишутся в чат 2 (manual)

### Requirement: Groups Chips
Транскрипт-реплики SHALL группироваться в чип «Расшифровка аудио (N)» (свёрнут: последняя реплика курсивом); вызовы quick-actions/tools — в чип «Инструменты (N)»; новые группы автосворачивают прежние (collapse_transcripts/collapse_operations default on).

#### Scenario: Чип транскрипта (test: manual_transcript_chip)
- WHEN пришли 5 turn
- THEN один чип «Расшифровка аудио (5)», клик разворачивает список (manual)

### Requirement: Status Indicator Window
Система SHALL показывать отдельное маленькое stealth-окно с бейджами: защита (вкл/откл), запись, автоответы, озвучка; угол = [ui] indicator_corner (default top-right).

#### Scenario: Бейджи (test: manual_indicator)
- WHEN включены автоответы и озвучка
- THEN в индикаторе зелёные бейджи; при «Защита откл.» — красный (manual)

## ADDED Requirements

### Requirement: Topbar Controls
Топбар SHALL содержать: mute; дропдаун STT (режим записи vad|manual, модель расшифровки, язык auto|ru|en); read-only модель LLM; «заметки» (dropdown списка заметок, клик = только просмотр в панели); «домой» (скрыть оверлей, показать главное окно).

#### Scenario: Заметки просмотр (test: manual_notes_view)
- WHEN выбрана заметка из dropdown
- THEN открывается панель с текстом заметки; в контекст LLM НЕ попадает (manual)

### Requirement: Toolbar Toggles
Нижняя тулбар SHALL содержать тумблеры с визуальным active-состоянием: скрыть чаты; активный контекст (dropdown созданных контекстов, применяет к активному чату); скриншот области; скриншот экрана; озвучка (mode auto/off); автоответы (trigger_mode auto/manual); функции ИИ (dropdown: «поиск в интернете», «использовать заметки»); сброс контекста (очищает историю активного чата).

#### Scenario: Тумблер автоответов (test: manual_auto_toggle)
- WHEN включён
- THEN речь интервьюера триггерит генерацию; выключен — нет (manual)

#### Scenario: Сброс контекста (test: manual_ctx_reset)
- WHEN нажат
- THEN лента и история активного чата пусты, следующий manual-запрос не содержит старых реплик (manual)

## REMOVED Requirements
(none)
