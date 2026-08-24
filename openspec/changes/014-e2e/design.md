# Design: E2E Runbook v2

## 1. Подготовка
1. Все changes 012–020 архивированы; `openspec/specs/` — source of truth.
2. config.toml: боевые ключи; hotkeys по умолчанию; tts.enabled=false (включить на части прогона).
3. Контексты: A (backend Rust), B (frontend + extra), C (livecoding python, languages=["ru","en"]).
4. Интервьюер: 15 teoria + 2 livecoding + 3 ловушки; OBS + Zoom-запись со стороны интервьюера.

## 2. Прогон (40 минут)
1. Создать встречу «E2E-1» + контекст A; «Продолжить».
2. Теория (15) + ловушки (3).
3. Переключить контекст на C; лайвкодинги со скриншотами (Ctrl+H / Ctrl+Shift+H).
4. В середине: Ctrl+W (click-through), Ctrl+B (hide/show), mute, «Резюме», ручной ввод.
5. Перезапуск приложения → «Продолжить» → проверить дописывание сессии.
6. Включить tts на 2 вопроса; переводы на лайвкодинге.

## 3. Пост-анализ
1. sqlite stats: p50/p95 ttft, answered/skipped/errors.
2. Точность: events.jsonl против разметки интервьюера (20 точек).
3. Stealth-тройка и ресурсы (Task Manager скриншоты до/после).
4. Replay-тюнинг: 3–4 набора параметров; выбор лучшего.

## 4. Acceptance gate
| Критерий | Порог |
|---|---|
| Релевантность ответов | ≥ 80% |
| SKIP-recall ловушек | ≥ 90% |
| p50 / p95 ttft | ≤ 1800 / 3000 мс |
| Stealth-тройка | 0 видимых захватов |
| CPU / RAM | ≤ 10% / ≤ 300МБ |
| UI-сценарии 014 | все manual зелёные |

Всё зелёное → `openspec archive 014-e2e` → релиз-кандидат.
Красный критерий → новый bugfix-change по OpenSpec, не ад-хок правки.
