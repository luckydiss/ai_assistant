# Design: Overlay 1:1

## 1. Config-structs (engine-config)

Добавить секции по спеке config (UiSection, WindowSection, ChatSection + поля stt.language, llm.search_enabled/search_tool_json) с serde-defaults и валидацией opacity/indicator_corner.

## 2. Store: таблицы

```sql
CREATE TABLE IF NOT EXISTS chats (
  id TEXT PRIMARY KEY, meeting_id TEXT NOT NULL, number INTEGER NOT NULL,
  context_id TEXT DEFAULT '');
CREATE TABLE IF NOT EXISTS notes (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, text TEXT DEFAULT '');
```
Методы: create_chat(meeting_id)->(id,number=MAX+1), list_chats(meeting_id), set_chat_context(chat_id, ctx_id), notes_list()->Vec<NoteRow>, note_get(id).

## 3. Orchestrator multi-chat

Inner: `chats: HashMap<String, ChatState>`, `active: String`, где ChatState { turns: Vec<Turn>, summary: String }.
- on_turn: self.chats.entry(active).or_default().turns.push(...)
- fire: брать ChatState активного чата.
- pub set_active_chat(&self, id), pub reset_active(&self) — через Cmd.
- trigger_mode читается при каждом on_turn из переданного при старте конфига (runtime-переключение: добавить Cmd::SetAuto(bool) — команда auto_answers_set шлёт его).

## 4. LLM search injection (client.rs)

```rust
// в stream_answer/complete после сборки body:
if self.search_enabled && !self.search_tool_json.is_empty() {
    if let Ok(extra) = serde_json::from_str::<serde_json::Value>(&self.search_tool_json) {
        if let (Some(b), Some(e)) = (body.as_object_mut(), extra.as_object()) {
            for (k, v) in e { b.insert(k.clone(), v.clone()); }
        }
    }
}
```
LlmClient::new принимает (search_enabled, search_tool_json).

## 5. STT language (client.rs stt)

В multipart: `if lang != "auto" { form = form.text("language", lang); }` — поле читать из конфига при старте пайплайна.

Справедливо. Даю **фронтенд-дополнение к 022** — три файла оверлея, заменяющие design §6–7. Агент реализует дословно; карта обработчиков из прошлого §7 остаётся для wiring.

## ui/overlay.html

```html
<!doctype html>
<html><head><meta charset="utf-8">
<link rel="stylesheet" href="overlay.css">
</head>
<body>
<!-- SVG-спрайт: все иконки 16x16, stroke=currentColor -->
<svg style="display:none" xmlns="http://www.w3.org/2000/svg">
  <symbol id="i-mic" viewBox="0 0 16 16"><rect x="6" y="2" width="4" height="8" rx="2"/><path d="M4 8a4 4 0 0 0 8 0M8 12v2" fill="none" stroke="currentColor" stroke-width="1.4"/></symbol>
  <symbol id="i-home" viewBox="0 0 16 16"><path d="M3 8l5-5 5 5M5 7v6h6V7" fill="none" stroke="currentColor" stroke-width="1.4"/></symbol>
  <symbol id="i-notes" viewBox="0 0 16 16"><rect x="4" y="2" width="8" height="12" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.4"/><path d="M6 5h4M6 8h4" stroke="currentColor" stroke-width="1.2"/></symbol>
  <symbol id="i-cam" viewBox="0 0 16 16"><rect x="2" y="5" width="8" height="7" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.4"/><path d="M10 8l4-2v5l-4-2" fill="none" stroke="currentColor" stroke-width="1.4"/></symbol>
  <symbol id="i-crop" viewBox="0 0 16 16"><path d="M3 6V3h3M13 10v3h-3M3 3l10 10" fill="none" stroke="currentColor" stroke-width="1.4"/></symbol>
  <symbol id="i-spk" viewBox="0 0 16 16"><path d="M3 6v4h3l4 3V3L6 6H3z" fill="none" stroke="currentColor" stroke-width="1.3"/><path d="M12 6a3 3 0 0 1 0 4" fill="none" stroke="currentColor" stroke-width="1.3"/></symbol>
  <symbol id="i-bolt" viewBox="0 0 16 16"><path d="M9 2L4 9h4l-1 5 5-7H8l1-5z" fill="none" stroke="currentColor" stroke-width="1.3"/></symbol>
  <symbol id="i-erase" viewBox="0 0 16 16"><path d="M4 10l6-6 3 3-6 6H4v-3zM4 13h9" fill="none" stroke="currentColor" stroke-width="1.4"/></symbol>
  <symbol id="i-chats" viewBox="0 0 16 16"><rect x="3" y="3" width="10" height="10" rx="2" fill="none" stroke="currentColor" stroke-width="1.4"/><path d="M6 3v10" stroke="currentColor" stroke-width="1.4"/></symbol>
  <symbol id="i-copy" viewBox="0 0 16 16"><rect x="5" y="5" width="8" height="8" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.3"/><path d="M11 5V3H3v8h2" fill="none" stroke="currentColor" stroke-width="1.3"/></symbol>
  <symbol id="i-del" viewBox="0 0 16 16"><path d="M3 4h10M6 4V3h4v1M5 4l1 9h4l1-9" fill="none" stroke="currentColor" stroke-width="1.3"/></symbol>
  <symbol id="i-mic-s" viewBox="0 0 16 16"><rect x="6" y="3" width="4" height="7" rx="2" fill="currentColor"/></symbol>
</svg>

<div id="app">
  <header id="topbar">
    <span id="logo">✳</span>
    <button class="ic" id="btnMute" title="Микрофон"><svg><use href="#i-mic"/></svg></button>
    <button class="ic dd" id="btnStt" title="Режим STT"><svg><use href="#i-mic"/></svg><span class="car">▾</span></button>
    <div id="drag"></div>
    <button class="pill" id="ddModel"><span class="dot"></span><span id="modelName">—</span><span class="car">▾</span></button>
    <button class="ic" id="btnNotes" title="Заметки"><svg><use href="#i-notes"/></svg></button>
    <button class="ic" id="btnHome" title="Домой"><svg><use href="#i-home"/></svg></button>
  </header>

  <div id="main">
    <nav id="rail"><div id="chats"></div><button class="ic" id="btnNewChat">+</button></nav>
    <div id="feed"></div>
  </div>

  <div id="qa">
    <button class="qa" id="qaAnalyze"><span class="ast">✳</span>Анализ экрана</button>
    <button class="qa" id="qaSay"><span class="ast">✳</span>Что сказать</button>
    <button class="qa" id="qaSummary"><span class="ast">✳</span>Резюме</button>
  </div>

  <div id="inputrow">
    <input id="input" placeholder="Сообщение или вопрос...">
    <button id="btnSend">↑</button>
  </div>

  <footer id="toolbar">
    <button class="ic tgl" id="btnChats" title="Чаты"><svg><use href="#i-chats"/></svg></button>
    <button class="pill" id="ddContext"><span id="ctxName">Контекст</span><span class="car">▾</span></button>
    <button class="ic" id="btnShotRegion" title="Скриншот области"><svg><use href="#i-crop"/></svg></button>
    <button class="ic" id="btnShotFull" title="Скриншот экрана"><svg><use href="#i-cam"/></svg></button>
    <button class="ic tgl" id="btnTts" title="Озвучка"><svg><use href="#i-spk"/></svg></button>
    <button class="ic tgl" id="btnAuto" title="Автоответы"><svg><use href="#i-bolt"/></svg></button>
    <button class="ic dd" id="btnAi" title="Функции ИИ"><svg><use href="#i-bolt"/></svg><span class="car">▾</span></button>
    <button class="ic" id="btnReset" title="Сбросить контекст"><svg><use href="#i-erase"/></svg></button>
  </footer>
</div>

<button id="fabDown">↓</button>

<div class="pane" id="paneStt" hidden>
  <label>Режим записи<select id="selSttMode"><option value="vad">Авто (VAD)</option><option value="manual">Ручной</option></select></label>
  <label>Модель<select id="selSttModel"><option>whisper-large-v3-turbo</option></select></label>
  <label>Язык<select id="selSttLang"><option value="auto">Авто</option><option value="ru">Русский</option><option value="en">English</option></select></label>
</div>
<div class="pane" id="paneNotes" hidden><div id="notesList"></div></div>
<div class="pane" id="paneCtx" hidden><div id="ctxList"></div></div>
<div class="pane" id="paneAi" hidden>
  <label class="chk"><input type="checkbox" id="tglSearch">Поиск в интернете</label>
  <label class="chk"><input type="checkbox" id="tglNotesRag">Использовать заметки</label>
</div>
<div id="notePane" hidden><pre id="noteText"></pre></div>

<script src="overlay.js"></script>
</body></html>
```

## ui/overlay.css

```css
:root{
  --accent:#f97316; --accent-dim:rgba(249,115,22,.15);
  --bg:16,16,18;            /* rgb поверхности */
  --surface:.92;            /* из [ui] opacity, ставится из JS */
  --panel:#1c1d21; --panel2:#212226; --line:#2a2b30;
  --tx:#e8e8ec; --tx2:#9a9aa3; --tx3:#6b6b74;
  --ok:#4ade80; --err:#f87171;
  --fs:13.5px; --r:12px;
}
*{box-sizing:border-box; margin:0}
html,body{background:transparent; overflow:hidden}
body{font:13px/1.45 system-ui,"Segoe UI",sans-serif; color:var(--tx)}
button{font:inherit; color:inherit; background:none; border:0; cursor:pointer}
svg{width:16px;height:16px; display:block}

#app{display:flex; flex-direction:column; height:100vh;
  background:rgba(var(--bg),var(--surface));
  border:1px solid var(--line); border-radius:14px; overflow:hidden}

/* ── топбар ─────────────────────────── */
#topbar{display:flex; align-items:center; gap:6px; padding:8px 10px;
  border-bottom:1px solid var(--line)}
#logo{color:var(--accent); font-size:18px; margin-right:2px}
#drag{flex:1; -webkit-app-region:drag}
.ic{width:32px;height:32px; border-radius:8px; display:grid; place-items:center;
  color:var(--tx2)}
.ic:hover{background:var(--panel2); color:var(--tx)}
.ic.tgl.on{color:var(--accent); background:var(--accent-dim)}
.ic .car,.pill .car{font-size:9px; color:var(--tx3); margin-left:3px}
.pill{display:flex; align-items:center; gap:7px; height:30px; padding:0 12px;
  border:1px solid var(--line); border-radius:999px; background:var(--panel);
  color:var(--tx2); font-size:12px}
.pill .dot{width:8px;height:8px;border-radius:50%;background:var(--tx3)}
.pill .dot.on{background:var(--ok)}

/* ── rail + лента ───────────────────── */
#main{flex:1; display:flex; min-height:0}
#rail{width:40px; border-right:1px solid var(--line); display:flex;
  flex-direction:column; align-items:center; padding:8px 0; gap:4px}
#rail.hidden{display:none}
#chats{flex:1; display:flex; flex-direction:column; gap:4px; align-items:center}
.chat-n{width:26px;height:26px; border-radius:7px; color:var(--tx3); font-size:12px;
  display:grid; place-items:center}
.chat-n.on{color:var(--accent); background:var(--accent-dim)}
#feed{flex:1; overflow-y:auto; padding:14px 16px; display:flex;
  flex-direction:column; gap:12px; font-size:var(--fs)}
#feed.top{flex-direction:column-reverse}

/* сообщения */
.m-user{align-self:flex-end; max-width:85%; background:var(--panel2);
  border-radius:12px; padding:8px 12px}
.m-quick{align-self:flex-end; color:var(--tx2); background:var(--panel);
  border:1px solid var(--line); border-radius:10px; padding:6px 10px; font-size:12px}
.m-quick::before{content:"› "; color:var(--tx3)}
.m-ai{display:flex; flex-direction:column; gap:6px}
.m-ai .hd{display:flex; align-items:center; gap:7px; color:var(--tx2); font-size:12px}
.m-ai .hd .av{width:16px;height:16px;border-radius:50%;background:var(--tx3)}
.m-ai .hd button{color:var(--tx3)} .m-ai .hd button:hover{color:var(--tx)}
.m-ai .bd{line-height:1.55}
.m-ai .bd b{color:var(--tx)}

/* группы-чипы */
.tg-h{color:var(--tx3); font-size:12px; background:none; padding:2px 0; text-align:left}
.tg-h:hover{color:var(--tx2)}
.tg-chip{display:flex; gap:8px; align-items:center; background:rgba(74,222,128,.08);
  border:1px solid rgba(74,222,128,.25); border-radius:10px; padding:7px 10px;
  color:var(--tx2); font-style:italic; overflow:hidden; white-space:nowrap}
.tg-chip svg{color:var(--ok); flex:none}
.tg-open{display:flex; flex-direction:column; gap:6px}
.tg-open .row{display:flex; gap:8px; color:var(--tx2)}
.tg-open .row svg{color:var(--ok); flex:none; margin-top:2px}

/* код */
.code{border-radius:10px; overflow:hidden; margin:6px 0; background:#101216}
.code-h{display:flex; justify-content:space-between; padding:6px 10px;
  color:var(--tx3); font-size:11px; background:#161a20}
.code pre{padding:10px 12px; overflow:auto; max-height:320px}
.code pre.noscroll{max-height:none}
.code code{font:12px/1.5 ui-monospace,Consolas,monospace; color:#d6d6dc}
.code .k{color:#ff7b72}.code .s{color:#a5d6ff}.code .t{color:#79c0fd}.code .f{color:#d2a8ff}

/* ── низ ───────────────────────────── */
#qa{display:flex; gap:8px; padding:8px 12px; justify-content:center}
.qa{display:flex; align-items:center; gap:7px; height:32px; padding:0 14px;
  border:1px solid var(--line); border-radius:999px; background:var(--panel);
  color:var(--tx2)}
.qa:hover{color:var(--tx); border-color:var(--tx3)}
.qa .ast{color:var(--accent)}
#inputrow{display:flex; gap:8px; padding:0 12px 10px}
#input{flex:1; height:38px; background:var(--panel); border:1px solid var(--line);
  border-radius:10px; padding:0 12px; color:var(--tx); outline:none}
#input:focus{border-color:var(--accent)}
#btnSend{width:36px;height:36px; border-radius:50%; background:var(--accent);
  color:#111; font-size:15px}
#toolbar{display:flex; gap:6px; padding:8px 10px; border-top:1px solid var(--line);
  align-items:center}

/* прочее */
#fabDown{position:fixed; right:14px; bottom:86px; width:38px;height:38px;
  border-radius:50%; background:var(--panel2); border:1px solid var(--line);
  color:var(--tx2)}
.pane{position:fixed; z-index:5; min-width:220px; background:var(--panel);
  border:1px solid var(--line); border-radius:10px; padding:8px;
  box-shadow:0 8px 24px rgba(0,0,0,.5); display:flex; flex-direction:column; gap:6px}
.pane label{display:flex; flex-direction:column; gap:4px; color:var(--tx2); font-size:12px}
.pane select{background:var(--panel2); color:var(--tx); border:1px solid var(--line);
  border-radius:8px; height:30px; padding:0 8px}
.pane .item{padding:6px 10px; border-radius:7px; color:var(--tx2); text-align:left}
.pane .item:hover{background:var(--panel2); color:var(--tx)}
.pane .chk{flex-direction:row; align-items:center; gap:8px}
#notePane{position:fixed; inset:60px 20px 120px; background:rgba(var(--bg),.97);
  border:1px solid var(--line); border-radius:12px; padding:14px; z-index:6}
#noteText{white-space:pre-wrap; color:var(--tx2); font-size:var(--fs)}
::-webkit-scrollbar{width:8px} ::-webkit-scrollbar-thumb{background:var(--line); border-radius:4px}
```

## ui/overlay.js (ключевые части)

```js
const { event, core } = window.__TAURI__;
const $ = s => document.querySelector(s);
const invoke = (n, a) => core.invoke(n, a);
const S = { chat: null, feed: [], curAi: null, curTg: null, cfg: null, autoscroll: true };

/* ── markdown (code-шапка, списки, bold) ── */
function esc(s){ return s.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;") }
function md(src){
  let out = "", parts = src.split("```");
  for (let i = 0; i < parts.length; i++){
    if (i % 2){                                   // код-блок
      const nl = parts[i].indexOf("\n");
      const lang = nl > -1 ? parts[i].slice(0, nl).trim() : "";
      const body = nl > -1 ? parts[i].slice(nl + 1) : parts[i];
      let h = esc(body)
        .replace(/\b(function|return|const|let|type|export|import|from|async|await|if|else|for|while)\b/g,'<span class="k">$1</span>')
        .replace(/("[^"]*"|'[^']*')/g,'<span class="s">$1</span>')
        .replace(/\b([A-Z][A-Za-z0-9_]*)\b/g,'<span class="t">$1</span>');
      out += `<div class="code"><div class="code-h"><span>${esc(lang)}</span><span>${S.cfg?.chat.code_theme||"github-dark"}</span></div><pre class="${S.cfg?.chat.code_scroll? "":"noscroll"}"><code>${h}</code></pre></div>`;
    } else {
      let t = esc(parts[i]);
      t = t.replace(/\*\*(.+?)\*\*/g,"<b>$1</b>").replace(/`([^`]+)`/g,"<code>$1</code>");
      t = t.split("\n").map(l => {
        const b = l.trim();
        if (b.startsWith("- "))  return `<div class="li">• ${b.slice(2)}</div>`;
        if (/^\d+\./.test(b))    return `<div class="li">${b}</div>`;
        return l;
      }).join("\n");
      out += t.replace(/\n{2,}/g,"</p><p>").replace(/\n/g,"<br>");
    }
  }
  return out;
}

/* ── рендер ленты ── */
function node(item){
  if (item.t === "user"){ const d = el("div","m-user"); d.textContent = item.text; return d; }
  if (item.t === "quick"){ const d = el("div","m-quick"); d.textContent = item.label; return d; }
  if (item.t === "tg"){
    const w = el("div","tg");
    const h = el("button","tg-h"); h.textContent = `↳ Расшифровка аудио (${item.items.length})`;
    h.onclick = () => { item.open = !item.open; renderFeed(); };
    w.appendChild(h);
    if (item.open){
      const o = el("div","tg-open");
      item.items.forEach(r => { const row = el("div","row");
        row.innerHTML = `<svg><use href="#i-mic-s"/></svg><span>${esc(r.text)}</span>`; o.appendChild(row); });
      w.appendChild(o);
    } else {
      const c = el("div","tg-chip"); const last = item.items[item.items.length-1];
      c.innerHTML = `<svg><use href="#i-mic-s"/></svg><span>${esc(last.text.slice(0,90))}…</span>`;
      w.appendChild(c);
    }
    return w;
  }
  if (item.t === "ai"){
    const w = el("div","m-ai");
    w.innerHTML = `<div class="hd"><span class="av"></span><span>${esc(item.model)}</span>
      <button data-a="copy"><svg><use href="#i-copy"/></svg></button>
      <button data-a="spk"><svg><use href="#i-spk"/></svg></button>
      <button data-a="del"><svg><use href="#i-del"/></svg></button></div>
      <div class="bd"></div>`;
    w.querySelector("[data-a=copy]").onclick = () => navigator.clipboard.writeText(item.buf);
    w.querySelector("[data-a=spk]").onclick  = () => invoke("tts_speak", { text: item.buf });
    w.querySelector("[data-a=del]").onclick  = () => { S.feed = S.feed.filter(x => x !== item); renderFeed(); };
    item.bd = w.querySelector(".bd"); item.bd.innerHTML = md(item.buf);
    return w;
  }
}
function el(t,c){ const d = document.createElement(t); d.className = c; return d }
function renderFeed(){
  const f = $("#feed"); f.innerHTML = "";
  S.feed.forEach(i => f.appendChild(node(i)));
  follow();
}
function follow(){ const f = $("#feed");
  if (S.autoscroll) f.scrollTo({ top: f.scrollHeight, behavior: S.cfg?.chat.autoscroll_speed > 66 ? "smooth" : "auto" }); }

/* ── события пайплайна ── */
event.listen("turn", e => {
  const { speaker, text } = e.payload;
  let tg = S.curTg;
  if (!tg || S.cfg?.chat.collapse_transcripts === false && false){}
  if (!tg){ tg = { t:"tg", items:[], open:false };
    if (S.cfg?.chat.collapse_transcripts) S.feed.forEach(i => { if (i.t==="tg") i.open = false; });
    S.feed.push(tg); S.curTg = tg; }
  tg.items.push({ speaker, text });
  renderFeed();
});
event.listen("answer_token", e => {
  if (!S.curAi){ S.curAi = { t:"ai", model: S.model || "assistant", buf:"" };
    S.feed.push(S.curAi); S.curTg = null; }
  S.curAi.buf += e.payload;
  requestAnimationFrame(() => { if (S.curAi?.bd) { S.curAi.bd.innerHTML = md(S.curAi.buf); follow(); } });
});
event.listen("answer_done", () => { S.curAi = null; });
event.listen("status", e => { if (e.payload === "generating") S.curTg = null; });

/* ── действия пользователя: логируем как quick ── */
function act(label, fn){ S.feed.push({ t:"quick", label }); renderFeed(); fn(); }
$("#qaSay").onclick     = () => act("Быстрое действие (Что сказать)", () => invoke("manual_trigger",{note:null}));
$("#qaSummary").onclick = () => act("Быстрое действие (Резюме)",      () => invoke("manual_trigger",{note:"сжато перескажи суть диалога"}));
$("#qaAnalyze").onclick = () => act("Быстрое действие (Разбор экрана)",() => invoke("screen_analyze",{windowOnly:false}));
$("#btnSend").onclick   = () => { const v = $("#input").value.trim(); if (!v) return;
  $("#input").value = ""; S.feed.push({ t:"user", text:v }); renderFeed(); invoke("manual_trigger",{note:v}); };

/* ── тумблеры/дропдауны (хелперы) ── */
function bindTgl(id, get, set){ const b = $(id);
  const up = () => b.classList.toggle("on", !!get());
  b.onclick = async () => { await set(!get()); up(); };
  b._up = up; return b; }
function bindDd(btnId, paneId){ const b = $(btnId), p = $(paneId);
  b.onclick = e => { e.stopPropagation(); document.querySelectorAll(".pane").forEach(x => x.hidden = true);
    const r = b.getBoundingClientRect(); p.style.top = (r.bottom + 6) + "px"; p.style.left = r.left + "px";
    p.hidden = !p.hidden; };
  document.addEventListener("click", () => p.hidden = true); }

/* ── init ── */
(async () => {
  S.cfg = await invoke("ui_get");                       // {accent,opacity,chat:{...},llm_model,...}
  document.documentElement.style.setProperty("--surface", (S.cfg.ui.opacity/100).toFixed(2));
  document.documentElement.style.setProperty("--accent", S.cfg.ui.accent);
  document.documentElement.style.setProperty("--fs", S.cfg.chat.font_size + "px");
  $("#modelName").textContent = S.cfg.llm_model;
  // тумблеры
  bindTgl("#btnAuto", () => S.auto,  v => invoke("auto_answers_set",{on:v}))._up();
  bindTgl("#btnTts",  () => S.tts,   v => invoke("tts_auto_set",{on:v}))._up();
  bindTgl("#btnChats",() => S.rail,  v => { $("#rail").classList.toggle("hidden", !v); return invoke("ui_set",{key:"rail",value:v}); });
  bindDd("#btnStt","#paneStt"); bindDd("#btnNotes","#paneNotes");
  bindDd("#ddContext","#paneCtx"); bindDd("#btnAi","#paneAi");
  // чаты
  S.chats = await invoke("chats_list"); renderChats();
  S.auto = (await invoke("auto_answers_get")); S.tts = (await invoke("tts_auto_get")); S.rail = true;
})();
function renderChats(){ const c = $("#chats"); c.innerHTML = "";
  S.chats.forEach(ch => { const d = el("button","chat-n" + (ch.id === S.chat ? " on":""));
    d.textContent = ch.number; d.onclick = async () => { S.chat = ch.id;
      await invoke("chat_switch",{id:ch.id}); S.feed = []; S.curAi = S.curTg = null;
      renderChats(); renderFeed(); };
    c.appendChild(d); }); }
$("#btnNewChat").onclick = async () => { const id = await invoke("chat_create");
  S.chats = await invoke("chats_list"); S.chat = id; renderChats(); };
```

---

**Ключевые моменты, которые агент не должен менять:** токены в `:root` (цвета/радиусы/шрифты как в собесе), прозрачность через `--surface` на `#app` (подложка просвечивает), rail 40px с номерами, чипы `tg-chip`/`tg-h` для групп транскрипта, quick-бабблы с `›`, код-блоки с шапкой и `max-height` (независимая прокрутка), тумблеры `.tgl.on` с акцентом, панes-дропдауны с позиционированием от кнопки.

Остальной wiring (команды, события, индикатор-окно) — из прошлого design §7–9 без изменений. Когда 022 будет зелёным — генерирую 023 (секции «Окно» и «Чат» с живыми превью, пишут в тот же конфиг).
## 8. Индикатор-окно (ui/indicator.html)

Маленькое окно 260x64, transparent, always-on-top, stealth-affinity; рендер бейджей из события "indicator" {protection, recording, auto, tts}; создаётся в setup: WebviewWindowBuilder url "indicator.html", позиция по [ui] indicator_corner.

## 9. Новые команды desktop

chats_list, chat_create, chat_switch, notes_list, note_get, stt_get, stt_set, ui_get, go_home, auto_answers_set, tts_auto_set, search_set, notes_rag_set, ctx_reset. Все — тонкие обёртки (store/config/orch), логика в engine.

## Рассмотрено и отклонено
- **Профили моделей в дропдауне:** отклонено (read-only)
- **RAG в этом change:** отклонено (флаг персистится, пайплайн в 024)
