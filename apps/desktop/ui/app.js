const { core } = window.__TAURI__;
const invoke = (n, a) => core.invoke(n, a);

const $ = s => document.querySelector(s);
const viewRoot = $("#view-root");
const sbEl = $("#sb");

function esc(s) {
  return String(s ?? "").replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function fmtDate(iso) {
  try {
    const d = new Date(iso);
    return d.toLocaleDateString("ru-RU", { day: "2-digit", month: "2-digit" }) + " " +
      d.toLocaleTimeString("ru-RU", { hour: "2-digit", minute: "2-digit" });
  } catch {
    return iso;
  }
}

/* ── toast ─────────────────────────── */
let toastTimer = null;
function toast(msg, isErr) {
  const t = $("#toast");
  t.textContent = msg;
  t.classList.toggle("err", !!isErr);
  t.hidden = false;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (t.hidden = true), isErr ? 4000 : 1800);
}

/* ── модалка ───────────────────────── */
function modal({ title, text = "", input = null, ok = "OK" }) {
  return new Promise(resolve => {
    const back = $("#modalBack");
    const inp = $("#modalInput");
    $("#modalTitle").textContent = title;
    const p = $("#modalText");
    p.textContent = text;
    p.hidden = !text;
    inp.value = input === null ? "" : input;
    inp.hidden = input === null;
    $("#modalOk").textContent = ok;
    back.hidden = false;
    if (input !== null) { inp.focus(); inp.select(); }

    const close = val => {
      back.hidden = true;
      $("#modalOk").onclick = null;
      $("#modalCancel").onclick = null;
      inp.onkeydown = null;
      resolve(val);
    };
    $("#modalOk").onclick = () => close(input === null ? true : inp.value.trim());
    $("#modalCancel").onclick = () => close(null);
    inp.onkeydown = e => {
      if (e.key === "Enter") $("#modalOk").click();
      if (e.key === "Escape") $("#modalCancel").click();
    };
  });
}
const confirmDlg = (title, text) => modal({ title, text, input: null, ok: "Удалить" });

/* ── конфиг ────────────────────────── */
async function cfgSet(section, key, value) {
  try {
    await invoke("config_set", { section, key, value });
    return true;
  } catch (e) {
    toast(String(e), true);
    return false;
  }
}

/* ── фабрики контролов ─────────────── */
function el(tag, cls) { const d = document.createElement(tag); if (cls) d.className = cls; return d; }

function setRow(title, desc, ctl) {
  const r = el("div", "set-row");
  const txt = el("div", "txt");
  txt.innerHTML = "<h2>" + esc(title) + "</h2>" + (desc ? '<div class="desc">' + esc(desc) + "</div>" : "");
  r.appendChild(txt);
  const c = el("div", "ctl");
  c.appendChild(ctl);
  r.appendChild(c);
  return r;
}

function mkSelect(options, value, onchange) {
  const s = document.createElement("select");
  for (const [val, label] of options)
    s.insertAdjacentHTML("beforeend", '<option value="' + esc(val) + '"' + (val === value ? " selected" : "") + ">" + esc(label) + "</option>");
  if (onchange) s.onchange = () => onchange(s.value);
  return s;
}

function mkToggle(on, onchange) {
  const b = el("button", "tgl-sw" + (on ? " on" : ""));
  b.setAttribute("aria-pressed", on);
  b.onclick = async () => {
    const next = !b.classList.contains("on");
    if (onchange && !await onchange(next)) return;
    b.classList.toggle("on", next);
  };
  return b;
}

function mkSlider(min, max, step, value, fmt, onchange) {
  const wrap = el("div", "sl-wrap");
  const inp = document.createElement("input");
  inp.type = "range";
  inp.min = min; inp.max = max; inp.step = step; inp.value = value;
  const badge = el("span", "sl-val");
  badge.textContent = fmt(Number(value));
  const paint = () => {
    inp.style.setProperty("--fill", ((inp.value - min) / (max - min)) * 100 + "%");
    badge.textContent = fmt(Number(inp.value));
  };
  paint();
  inp.oninput = paint;
  inp.onchange = () => onchange && onchange(Number(inp.value));
  wrap.appendChild(inp);
  wrap.appendChild(badge);
  return wrap;
}

const ACCENT_PRESETS = ["#f97316", "#3b82f6", "#8b5cf6", "#10b981", "#ec4899"];
function mkSwatches(value, onchange) {
  const w = el("div", "swatches");
  for (const c of ACCENT_PRESETS) {
    const b = el("button", "swatch" + (c.toLowerCase() === String(value).toLowerCase() ? " on" : ""));
    b.style.background = c;
    b.title = c;
    b.onclick = () => { w.querySelectorAll(".swatch").forEach(x => x.classList.remove("on")); b.classList.add("on"); onchange(c); };
    w.appendChild(b);
  }
  const custom = el("button", "sw-custom");
  custom.innerHTML = '<input type="color" value="' + esc(/^#[0-9a-f]{6}$/i.test(value) ? value : "#f97316") + '"><span>Свой</span>';
  const picker = custom.querySelector("input");
  picker.onclick = e => e.stopPropagation();
  custom.onclick = () => picker.click();
  picker.oninput = () => {
    w.querySelectorAll(".swatch").forEach(x => x.classList.remove("on"));
    onchange(picker.value);
  };
  w.appendChild(custom);
  return w;
}

/* превью чата */
function chatPreview(fontSize) {
  const p = el("div", "preview");
  p.innerHTML =
    '<div class="pv-head"><span>Предпросмотр чата</span><span>' + esc(fontSize) + ' px</span></div>' +
    '<div class="pv-user" style="font-size:' + fontSize + 'px">Объясни решение коротко и по шагам.</div>' +
    '<div class="pv-ai-name"><i>✳</i>GPT Smart</div>' +
    '<div class="pv-ai-text" style="font-size:' + fontSize + 'px">Сначала определим входные данные, затем выберем алгоритм и оценим его сложность.</div>';
  return p;
}

function codePreview(theme) {
  const p = el("div", "preview");
  p.innerHTML =
    '<div class="pv-head"><span>Предпросмотр кода</span><span>' + esc(theme) + "</span></div>" +
    '<div class="pv-code"><div class="pv-code-h"><span>preview.ts</span><span>' + esc(theme) + "</span></div>" +
    '<pre><span class="k">type</span> <span class="t">Message</span> = {<br>  role: <span class="s">"user"</span> | <span class="s">"assistant"</span>;<br>  text: <span class="t">string</span>;<br>};</pre></div>';
  return p;
}

function collapsePreview(label) {
  const p = el("div", "preview");
  p.innerHTML =
    '<div class="pv-head"><span>Пример после новых сообщений</span><span>свернуто</span></div>' +
    '<div class="pv-tg"><i>↳</i> Расшифровка аудио (5) — Можешь объяснить, как работает Event Loop? Сначала выполняется синхронный код…</div>' +
    '<div class="pv-ai-name"><i>✳</i>GPT Smart</div>' +
    '<div class="pv-ai-text">Event Loop сначала выполняет стек вызовов, затем microtasks, а после этого берёт следующую макрозадачу.</div>';
  return p;
}

/* ── сайдбар настроек ──────────────── */
const ICONS = {
  audio: '<svg viewBox="0 0 16 16"><rect x="6" y="2" width="4" height="8" rx="2" fill="none" stroke="currentColor" stroke-width="1.4"/><path d="M4 8a4 4 0 0 0 8 0M8 12v2" fill="none" stroke="currentColor" stroke-width="1.4"/></svg>',
  tts: '<svg viewBox="0 0 16 16"><path d="M3 6v4h3l4 3V3L6 6H3z" fill="none" stroke="currentColor" stroke-width="1.3"/><path d="M12 6a3 3 0 0 1 0 4" fill="none" stroke="currentColor" stroke-width="1.3"/></svg>',
  hotkeys: '<svg viewBox="0 0 16 16"><rect x="1.5" y="4" width="13" height="8" rx="2" fill="none" stroke="currentColor" stroke-width="1.3"/><path d="M4 7h.01M6.5 7h.01M9 7h.01M11.5 7h.01M4.5 9.5h7" stroke="currentColor" stroke-width="1.4"/></svg>',
  chat: '<svg viewBox="0 0 16 16"><path d="M2 3.5A1.5 1.5 0 0 1 3.5 2h9A1.5 1.5 0 0 1 14 3.5v7a1.5 1.5 0 0 1-1.5 1.5H8l-3.5 3v-3h-1A1.5 1.5 0 0 1 2 10.5z" fill="none" stroke="currentColor" stroke-width="1.3"/></svg>',
  window: '<svg viewBox="0 0 16 16"><rect x="2" y="2.5" width="12" height="11" rx="2" fill="none" stroke="currentColor" stroke-width="1.4"/><path d="M2 5.5h12" stroke="currentColor" stroke-width="1.4"/></svg>',
  protection: '<svg viewBox="0 0 16 16"><path d="M8 1.5l5 2v4c0 3.2-2.1 5.7-5 7-2.9-1.3-5-3.8-5-7v-4z" fill="none" stroke="currentColor" stroke-width="1.4"/></svg>',
};

const SB_GROUPS = [
  { group: "Запись", items: [["audio", "Запись звука"]] },
  { group: "Звук", items: [["tts", "Озвучка ответов"]] },
  { group: "Управление", items: [["hotkeys", "Горячие клавиши"]] },
  { group: "Интерфейс", items: [["chat", "Чат"], ["window", "Окно"], ["protection", "Защита"]] },
];

function renderSidebar(active) {
  const nav = $("#sb-nav");
  nav.innerHTML = "";
  for (const g of SB_GROUPS) {
    nav.insertAdjacentHTML("beforeend", '<div class="sb-group">' + esc(g.group) + "</div>");
    for (const [id, label] of g.items) {
      const b = el("button", "sb-item" + (id === active ? " on" : ""));
      b.innerHTML = (ICONS[id] || "") + "<span>" + esc(label) + "</span>";
      b.onclick = () => { location.hash = "#settings/" + id; };
      nav.appendChild(b);
    }
  }
}

/* ── секции настроек ───────────────── */
function pageHead(title, desc) {
  viewRoot.innerHTML = "";
  viewRoot.insertAdjacentHTML("beforeend", "<h1>" + esc(title) + "</h1>");
  if (desc) viewRoot.insertAdjacentHTML("beforeend", '<p class="page-desc">' + esc(desc) + "</p>");
}

async function secChat(cfg) {
  pageHead("Чат", "Внешний вид и поведение ленты сообщений оверлея.");
  const c = cfg.chat;

  const rowOrder = setRow("Порядок сообщений", "Выберите, у какого края чата появляются новые сообщения.",
    mkSelect([["bottom", "Новые снизу"], ["top", "Новые сверху"]], c.order,
      v => cfgSet("chat", "order", v)));
  viewRoot.appendChild(rowOrder);

  const pv = chatPreview(c.font_size);
  const rowFont = setRow("Размер текста сообщений", "Изменяет размер основного текста сообщений и расшифровок в чате.",
    mkSlider(11, 18, 0.5, c.font_size, v => v + " px", async v => { if (!await cfgSet("chat", "font_size", v)) return; }));
  rowFont.classList.add("card"); rowFont.style.display = "block";
  rowFont.querySelector(".ctl").style.marginTop = "12px";
  rowFont.appendChild(pv);
  const slider = rowFont.querySelector("input[type=range]");
  slider.addEventListener("input", () => {
    const fs = Number(slider.value);
    rowFont.querySelector(".pv-user").style.fontSize = fs + "px";
    rowFont.querySelector(".pv-ai-text").style.fontSize = fs + "px";
    rowFont.querySelector(".pv-head span:last-child").textContent = fs + " px";
  });
  viewRoot.appendChild(rowFont);

  const pvTheme = codePreview(c.code_theme);
  const rowTheme = setRow("Тема подсветки кода", "Выберите тему для блоков кода в ответах и заметках.",
    mkSelect([["github-dark", "GitHub Dark"], ["monokai", "Monokai"]], c.code_theme,
      v => { cfgSet("chat", "code_theme", v); pvTheme.remove(); rowTheme.appendChild(codePreview(v)); }));
  rowTheme.classList.add("card"); rowTheme.style.display = "block";
  rowTheme.querySelector(".ctl").style.marginTop = "10px";
  rowTheme.appendChild(pvTheme);
  viewRoot.appendChild(rowTheme);

  viewRoot.appendChild(setRow("Независимая прокрутка длинного кода",
    "Ограничивать высоту длинных блоков кода, прокручивать их отдельно от чата и разрешать полный разворот.",
    mkToggle(c.code_scroll, v => cfgSet("chat", "code_scroll", v))));

  viewRoot.appendChild(setRow("Автопрокрутка чата",
    "Автоматически следовать за краем новых сообщений. Отключается, когда вы уходите в историю.",
    mkToggle(c.autoscroll, v => cfgSet("chat", "autoscroll", v))));

  viewRoot.appendChild(setRow("Скорость автопрокрутки",
    "Регулирует, как быстро чат следует за новыми сообщениями.",
    mkSlider(0, 100, 5, c.autoscroll_speed, v => v + "%", v => cfgSet("chat", "autoscroll_speed", v))));

  const rowTg = setRow("Автоматически сворачивать расшифровки до самой новой",
    "Группы расшифровок, кроме самой новой, автоматически сворачиваются для компактного отображения.",
    mkToggle(c.collapse_transcripts, v => cfgSet("chat", "collapse_transcripts", v)));
  viewRoot.appendChild(rowTg);
  viewRoot.appendChild(collapsePreview());

  viewRoot.appendChild(setRow("Сворачивать последние группы тоже",
    "Автосворачивание будет работать и для самой новой группы.",
    mkToggle(c.collapse_last, v => cfgSet("chat", "collapse_last", v))));

  viewRoot.appendChild(setRow("Компактные быстрые действия",
    "Показывать в чате короткую строку вместо полного текста подсказки. Полный текст можно раскрыть.",
    mkToggle(c.compact_quick, v => cfgSet("chat", "compact_quick", v))));

  viewRoot.appendChild(setRow("Отменять генерацию при повторной отправке",
    "Если вы отправляете новое сообщение или быстрое действие, пока ИИ ещё отвечает, текущая генерация отменяется и стартует новая.",
    mkToggle(c.cancel_on_resend, v => cfgSet("chat", "cancel_on_resend", v))));

  viewRoot.appendChild(setRow("Режим ручной отмены генерации",
    "Что делать с уже полученным текстом, когда вы нажимаете отмену генерации.",
    mkSelect([["drop", "Отменить без сохранения"], ["keep", "Сохранить частичный текст"]], c.cancel_mode,
      v => cfgSet("chat", "cancel_mode", v))));
}

async function secWindow(cfg) {
  pageHead("Окно", "Акцент, прозрачность и поведение окна оверлея.");
  const u = cfg.ui;

  viewRoot.appendChild(setRow("Акцентный цвет",
    "Выберите базовый цвет для кнопок, выделений и активных состояний.",
    mkSwatches(u.accent, v => cfgSet("ui", "accent", v))));

  viewRoot.appendChild(setRow("Непрозрачность интерфейса",
    "Настройте прозрачность оверлея во время встречи.",
    mkSlider(10, 100, 5, u.opacity, v => v + "%", v => cfgSet("ui", "opacity", v))));

  viewRoot.appendChild(setRow("Шаг перемещения окна",
    "Количество пикселей, на которое окно перемещается при нажатии горячих клавиш.",
    mkSlider(10, 200, 10, cfg.window.move_step, v => v + " px",
      v => cfgSet("window", "move_step", v))));

  viewRoot.appendChild(setRow("Шаг изменения размера окна",
    "Количество пикселей, на которое изменяется размер окна при нажатии горячих клавиш.",
    mkSlider(10, 200, 10, cfg.window.resize_step, v => v + " px",
      v => cfgSet("window", "resize_step", v))));
}

async function secProtection(cfg) {
  pageHead("Защита", "Маскировка окна от демонстрации и записи экрана.");
  const on = !!(cfg.ui && cfg.ui.protection);
  const row = setRow("Режим защиты",
    "Окно оверлея не видно в демонстрации экрана, записи видео и скриншотах (Windows).",
    mkToggle(on, async v => {
      try {
        await invoke("protection_set", { on: v });
        updateProtBadge(v);
        toast(v ? "Защита включена" : "Защита выключена");
        return true;
      } catch (e) { toast(String(e), true); return false; }
    }));
  viewRoot.appendChild(row);
}

async function secAudio() {
  pageHead("Запись звука", "Источник аудио и режим распознавания речи.");
  const card = el("div", "card");
  card.innerHTML = "<h2>Источник</h2><div class=\"desc\">Откуда брать звук для расшифровки.</div>";
  const [devices, cfg] = await Promise.all([
    invoke("list_audio_devices").catch(() => []),
    invoke("get_config").catch(() => null),
  ]);
  const wrap = el("div");
  wrap.style.marginTop = "12px";
  wrap.innerHTML = "";
  const selSource = mkSelect([["system+mic", "Система + микрофон"], ["system", "Только система"], ["mic", "Только микрофон"]], "system+mic");
  const selMode = mkSelect([["vad", "Авто (VAD)"], ["manual", "Ручной"]], "vad");
  const selMic = mkSelect([["", "по умолчанию"], ...devices.map(d => [d, d])], "");
  if (cfg) {
    selSource.value = cfg.audio.source || "system+mic";
    selMode.value = cfg.audio.mode || "manual";
    if (cfg.audio.micDevice) selMic.value = cfg.audio.micDevice;
  }
  wrap.insertAdjacentHTML("beforeend", '<div class="m-label" style="margin-top:0">Каналы записи</div>');
  wrap.appendChild(selSource);
  wrap.insertAdjacentHTML("beforeend", '<div class="m-label">Режим записи</div>');
  wrap.appendChild(selMode);
  wrap.insertAdjacentHTML("beforeend", '<div class="m-label">Микрофон</div>');
  wrap.appendChild(selMic);
  const save = el("button", "primary");
  save.textContent = "Сохранить";
  save.style.marginTop = "16px";
  save.onclick = async () => {
    try {
      await invoke("update_audio_settings", {
        source: selSource.value,
        mode: selMode.value,
        micDevice: selMic.value || null,
      });
      toast("Настройки записи сохранены");
    } catch (e) { toast(String(e), true); }
  };
  wrap.appendChild(save);
  card.appendChild(wrap);
  viewRoot.appendChild(card);
}

async function secTts() {
  pageHead("Озвучка ответов", "Синтез речи ответов ИИ (Cartesia).");
  const cfg = await invoke("get_config").catch(() => null);
  const row = setRow("Режим озвучки",
    "Авто — озвучка включается сразу; по хоткею — только при нажатии Ctrl+T.",
    mkSelect([["off", "Выкл"], ["auto", "Авто (стриминг)"], ["hotkey", "По хоткею (Ctrl+T)"]],
      cfg ? (cfg.tts && cfg.tts.mode) || "off" : "off",
      async v => {
        try { await invoke("tts_set_mode", { mode: v }); toast("Режим озвучки сохранён"); }
        catch (e) { toast(String(e), true); }
      }));
  viewRoot.appendChild(row);
  viewRoot.insertAdjacentHTML("beforeend", '<p class="hint">Ключ API задаётся в config.toml ([tts] api_key)</p>');
}

async function secHotkeys() {
  pageHead("Горячие клавиши", "Глобальные сочетания клавиш. Изменения применяются автоматически.");
  const hk = await invoke("hotkeys_get").catch(() => null);
  const ACTIONS = [
    ["manual", "Что сказать"],
    ["hide", "Скрыть оверлей"],
    ["click_through", "Click-through"],
    ["mute", "Mute"],
    ["record", "Запись"],
    ["screenshot_full", "Скриншот (весь)"],
    ["screenshot_region", "Скриншот (регион)"],
  ];
  const card = el("div", "card");
  for (const [action, label] of ACTIONS) {
    const row = el("div", "hk-row");
    row.innerHTML = "<span>" + esc(label) + "</span>" +
      '<input type="text" data-accel="' + action + '" value="' + esc(hk ? hk[action] : "") + '" placeholder="(пусто = отключено)">' +
      '<span class="hk-status" data-status="' + action + '"></span>';
    card.appendChild(row);
  }
  viewRoot.appendChild(card);

  let seq = 0;
  card.querySelectorAll("[data-accel]").forEach(inp => {
    const status = card.querySelector('[data-status="' + inp.dataset.accel + '"]');
    inp.onchange = async () => {
      const cur = ++seq;
      try {
        await invoke("set_hotkey", { action: inp.dataset.accel, accel: inp.value.trim() });
        if (cur === seq) { status.textContent = "✓"; setTimeout(() => (status.textContent = ""), 1500); }
      } catch (e) {
        status.textContent = "!";
        status.style.color = "var(--err)";
        toast(String(e), true);
        setTimeout(() => { status.textContent = ""; status.style.color = ""; }, 3000);
      }
    };
    inp.onkeydown = e => { if (e.key === "Enter") inp.blur(); };
  });
}

/* ── встречи ───────────────────────── */
async function viewMeetings() {
  sbEl.classList.add("hidden");
  setRailActive("meetings");
  pageHead("Встречи", "Создайте встречу и начните запись — оверлей откроется автоматически.");
  const [meetings, contexts] = await Promise.all([
    invoke("meetings_list").catch(() => []),
    invoke("contexts_list").catch(() => []),
  ]);
  const v = viewRoot;

  const form = el("div", "card");
  form.innerHTML = "<h2>Новая встреча</h2>" +
    '<div class="form-row" style="margin-top:12px"><input type="text" id="m-name" placeholder="Название">' +
    '<input type="text" id="m-vacancy" placeholder="Вакансия" style="max-width:220px"></div>' +
    '<div class="form-row"><button id="m-create" class="primary">Создать и начать</button></div>';
  v.appendChild(form);

  form.querySelector("#m-create").onclick = async () => {
    const name = form.querySelector("#m-name").value.trim();
    const vacancy = form.querySelector("#m-vacancy").value.trim();
    if (!name) { toast("Укажите название встречи", true); return; }
    const id = await invoke("meeting_create", { name, vacancy });
    await viewMeetings();
    startMeeting(id);
  };

  if (!meetings.length) {
    v.insertAdjacentHTML("beforeend", '<div class="card"><div class="empty-note">Пока нет встреч — создайте первую.</div></div>');
  }

  for (const m of meetings) {
    const card = el("div", "card");
    const ctxSelect = '<select data-mid="' + m.id + '">' +
      '<option value="">— без контекста —</option>' +
      contexts.map(c =>
        '<option value="' + c.id + '"' + (m.context_id === c.id ? " selected" : "") + ">" +
        esc(c.name) + "</option>"
      ).join("") + "</select>";
    card.innerHTML =
      '<div class="title" style="font-weight:650">' + esc(m.name) + "</div>" +
      '<div class="desc">' + fmtDate(m.created_at) +
      (m.vacancy ? " · " + esc(m.vacancy) : "") +
      " · сообщений: " + m.messages + "</div>" +
      '<div class="form-row">' + ctxSelect +
      '<button data-continue="' + m.id + '" class="primary">Продолжить</button>' +
      '<button data-rename="' + m.id + '">Переименовать</button>' +
      '<button data-delete="' + m.id + '" class="danger">Удалить</button></div>';
    v.appendChild(card);

    card.querySelector("[data-continue]").onclick = () => startMeeting(m.id);
    card.querySelector("[data-delete]").onclick = async () => {
      const yes = await confirmDlg("Удалить встречу?", "«" + m.name + "» будет удалена вместе с историей сообщений.");
      if (!yes) return;
      await invoke("meeting_delete", { id: m.id });
      await viewMeetings();
    };
    card.querySelector("[data-rename]").onclick = async () => {
      const name = await modal({ title: "Переименовать встречу", input: m.name, ok: "Сохранить" });
      if (name && name !== m.name) {
        await invoke("meeting_rename", { id: m.id, name });
        await viewMeetings();
      }
    };
    card.querySelector("select").onchange = async (e) => {
      await invoke("meeting_set_context", { meetingId: m.id, contextId: e.target.value });
    };
  }
}

function startMeeting(id) {
  invoke("start_pipeline", { meetingId: id }).catch(e => toast(String(e), true));
}

/* ── контексты ─────────────────────── */
async function viewContexts() {
  sbEl.classList.add("hidden");
  setRailActive("contexts");
  pageHead("Контексты", "Резюме, роль и промпт для конкретной вакансии — используются в подсказках.");
  const contexts = await invoke("contexts_list").catch(() => []);
  const v = viewRoot;

  const form = el("div", "card");
  form.innerHTML = '<h2 id="c-form-title">Новый контекст</h2>' +
    '<label class="m-label" style="margin-top:12px">Имя</label><input type="text" id="c-name">' +
    '<label class="m-label">Роль</label><input type="text" id="c-role" placeholder="Например: Senior Backend разработчик">' +
    '<div class="hint">Кем ты выступаешь на собеседовании — ИИ будет отвечать от этого лица и в этом стеке</div>' +
    '<label class="m-label">Языки (через запятую)</label><input type="text" id="c-langs" value="ru, en">' +
    '<label class="m-label">Резюме (TXT/MD)</label><textarea id="c-resume" rows="5"></textarea>' +
    '<div class="form-row"><button id="c-import">Загрузить файл</button>' +
    '<input type="file" id="c-file" accept=".txt,.md" class="hidden"></div>' +
    '<label class="m-label">Промпт (инструкция для ответов)</label><textarea id="c-extra" rows="2"></textarea>' +
    '<div class="form-row"><button id="c-save" class="primary">Сохранить</button>' +
    '<button id="c-cancel-edit" class="mini hidden">Отменить редактирование</button></div>';
  v.appendChild(form);

  const saveBtn = form.querySelector("#c-save");
  const cancelBtn = form.querySelector("#c-cancel-edit");
  let editing = null;

  function readForm(id) {
    return {
      id,
      name: form.querySelector("#c-name").value.trim(),
      role: form.querySelector("#c-role").value.trim(),
      languages: form.querySelector("#c-langs").value.split(",").map(s => s.trim()).filter(Boolean),
      resumeText: form.querySelector("#c-resume").value,
      extraPrompt: form.querySelector("#c-extra").value,
    };
  }
  function syncMode() {
    form.querySelector("#c-form-title").textContent =
      editing ? "Редактирование: " + editing.name : "Новый контекст";
    saveBtn.textContent = editing ? "Сохранить изменения" : "Сохранить";
    cancelBtn.classList.toggle("hidden", !editing);
    if (!editing) {
      form.querySelector("#c-name").value = "";
      form.querySelector("#c-role").value = "";
      form.querySelector("#c-langs").value = "ru, en";
      form.querySelector("#c-resume").value = "";
      form.querySelector("#c-extra").value = "";
    }
  }
  cancelBtn.onclick = () => { editing = null; syncMode(); };

  form.querySelector("#c-import").onclick = () => form.querySelector("#c-file").click();
  form.querySelector("#c-file").onchange = async (e) => {
    const f = e.target.files[0];
    if (!f) return;
    form.querySelector("#c-resume").value = await f.text();
  };

  saveBtn.onclick = async () => {
    const ctx = readForm(editing ? editing.id : crypto.randomUUID());
    if (!ctx.name) { toast("Укажите имя контекста", true); return; }
    try {
      await invoke("context_save", { ctx });
      toast(editing ? "Контекст обновлён" : "Контекст сохранён");
      editing = null;
      await viewContexts();
    } catch (e) {
      toast(String(e), true);
    }
  };

  if (!contexts.length) {
    v.insertAdjacentHTML("beforeend", '<div class="card"><div class="empty-note">Нет контекстов.</div></div>');
  }

  for (const c of contexts) {
    const card = el("div", "card");
    card.innerHTML =
      '<div class="title" style="font-weight:650">' + esc(c.name) + "</div>" +
      '<div class="desc">' + (c.role ? esc(c.role) : "—") + "</div>" +
      '<div class="form-row"><button data-edit="' + c.id + '">Редактировать</button> ' +
      '<button data-del="' + c.id + '" class="danger">Удалить</button></div>';
    v.appendChild(card);

    card.querySelector("[data-del]").onclick = async () => {
      const yes = await confirmDlg("Удалить контекст?", "«" + c.name + "» будет удалён безвозвратно.");
      if (!yes) return;
      await invoke("context_delete", { id: c.id });
      if (editing && editing.id === c.id) { editing = null; syncMode(); }
      await viewContexts();
    };
    card.querySelector("[data-edit]").onclick = () => {
      editing = c;
      form.querySelector("#c-name").value = c.name;
      form.querySelector("#c-role").value = c.role || "";
      form.querySelector("#c-langs").value = (c.languages || []).join(", ");
      form.querySelector("#c-resume").value = c.resumeText || "";
      form.querySelector("#c-extra").value = c.extraPrompt || "";
      syncMode();
      form.scrollIntoView({ behavior: "smooth", block: "start" });
    };
  }
}

/* ── роутер ────────────────────────── */
function setRailActive(name) {
  for (const id of ["meetings", "contexts", "settings"])
    $("#rl-" + id).classList.toggle("on", id === name);
}

function updateProtBadge(on) {
  const b = $("#tb-prot");
  b.className = on ? "badge-on" : "badge-off";
  b.querySelector("span").textContent = on ? "Защита вкл." : "Защита откл.";
}

async function route() {
  const hash = location.hash.replace("#", "") || "meetings";
  const [root, sub] = hash.split("/");
  if (root === "contexts") { viewContexts(); return; }
  if (root === "settings") {
    setRailActive("settings");
    sbEl.classList.remove("hidden");
    const section = ["audio", "tts", "hotkeys", "chat", "window", "protection"].includes(sub) ? sub : "chat";
    renderSidebar(section);
    const cfg = await invoke("get_config").catch(() => null);
    updateProtBadge(!!(cfg && cfg.ui && cfg.ui.protection));
    const renderers = {
      audio: secAudio, tts: secTts, hotkeys: secHotkeys,
      chat: secChat, window: secWindow, protection: secProtection,
    };
    if (cfg) await renderers[section](cfg);
    else viewRoot.innerHTML = '<div class="card"><div class="empty-note">Конфигурация недоступна</div></div>';
    return;
  }
  viewMeetings();
}

$("#rl-meetings").onclick = () => { location.hash = "#meetings"; };
$("#rl-contexts").onclick = () => { location.hash = "#contexts"; };
$("#rl-settings").onclick = () => { location.hash = "#settings/chat"; };
$("#sb-close").onclick = () => { location.hash = "#meetings"; };
$("#tb-prot").onclick = () => { location.hash = "#settings/protection"; };
$("#tb-refresh").onclick = () => route();
$("#tb-continue").onclick = async () => {
  const meetings = await invoke("meetings_list").catch(() => []);
  if (!meetings.length) { toast("Нет встреч для продолжения", true); return; }
  startMeeting(meetings[0].id);
};

window.addEventListener("hashchange", route);
route();
