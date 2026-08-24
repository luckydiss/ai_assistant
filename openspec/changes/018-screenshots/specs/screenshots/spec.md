# Delta: Screenshots

## ADDED Requirements

### Requirement: Screen Capture
Система SHALL снимать весь экран и активное окно через GDI и кодировать в PNG.

#### Scenario: PNG валиден (test: capture_produces_png)
- GIVEN вызов capture_screen()
- WHEN закодировано
- THEN байты начинаются с 0x89 'P' 'N' 'G' и размеры > 0

#### Scenario: Кроп активного окна (test: crop_window_region)
- GIVEN полноэкранный буфер и rect (100,100,200,150)
- WHEN crop()
- THEN результат 200x150, пиксели совпадают с исходной областью

### Requirement: Vision Request
Система SHALL прикреплять скриншот к запросу как image_url (data URI base64) и LLM-запрос SHALL содержать мультимодальную часть.

#### Scenario: image_url в запросе (test: vision_payload_sent)
- GIVEN manual(note, Some(b64)) и mock-сервер
- WHEN запрос ушёл
- THEN тело содержит "image_url" и "data:image/png;base64,"

#### Scenario: Анализ экрана (test: manual_screen_analyze)
- WHEN нажат Ctrl+H или кнопка «Анализ экрана»
- THEN в чате ответ, учитывающий содержимое экрана (manual)

### Requirement: Screenshot Hotkeys
Ctrl+H SHALL прикреплять полный скриншот к следующему запросу; Ctrl+Shift+H — скриншот активного окна.

#### Scenario: Хоткей окна (test: manual_window_shot)
- WHEN фокус в IDE и нажат Ctrl+Shift+H
- THEN ответ по коду из IDE (manual)

## MODIFIED Requirements
(none)

## REMOVED Requirements
(none)
