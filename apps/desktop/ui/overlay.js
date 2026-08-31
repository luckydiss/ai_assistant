const { event, core } = window.__TAURI__;
const $ = s => document.querySelector(s);
const invoke = (n, a) => core.invoke(n, a);
const S = { chat: null, feeds: {}, loaded: {}, feed: [], curAi: null, curTg: null, cfg: null, autoscroll: true, modelDisplay: "Assistant" };
let rafPending = false;

/* ── toast ─────────────────────────── */
let toastTimer = null;
function toast(msg, isErr){
  let t = $("#toast");
  if (!t){
    t = document.createElement("div");
    t.id = "toast";
    document.body.appendChild(t);
  }
  t.textContent = msg;
  t.classList.toggle("err", !!isErr);
  t.hidden = false;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (t.hidden = true), isErr ? 4000 : 2500);
}

/* ── markdown ───────────────────────── */
let cbId = 0;
function esc(s){ return String(s ?? "").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;") }
function escAttr(s){ return String(s ?? "").replace(/"/g,"&quot;") }
function inlineMd(s){
  return esc(s)
    .replace(/\*\*([^*\n][\s\S]*?[^*\n])\*\*/g,"<b>$1</b>")
    .replace(/(^|[\s(«])\*([^*\n]+?)\*(?=[\s).,!?:;»]|$)/g,"$1<b>$2</b>")
    .replace(/`([^`\n]+?)`/g,"<code>$1</code>");
}
function lineToHtml(line){
  const b = line.trim();
  if (!b) return "";
  if (/^[-•]\s+/.test(b)) return `<div class="li">• ${inlineMd(b.replace(/^[-•]\s+/,""))}</div>`;
  if (/^\d+[.)]\s+/.test(b)) return `<div class="li">${inlineMd(b)}</div>`;
  return inlineMd(line) + "<br>";
}
function md(src){
  src = String(src ?? "");
  const parts = src.split("```");
  let out = "";
  for (let i = 0; i < parts.length; i++){
    if (i % 2 === 1){
      const raw = parts[i];
      const nl = raw.indexOf("\n");
      const lang = nl >= 0 ? raw.slice(0, nl).trim() : "";
      const body = nl >= 0 ? raw.slice(nl + 1) : raw;
      out += codeBlock(lang, body);
      continue;
    }
    let t = parts[i];
    const paragraphs = t
      .split(/\n{2,}/)
      .map(p => p.trim())
      .filter(Boolean)
      .map(p => {
        const lines = p.split("\n").map(lineToHtml).join("");
        return `<p>${lines}</p>`;
      });
    out += paragraphs.join("");
  }
  return out;
}
function codeBlock(lang, body){
  const id = "cb" + (++cbId);
  const theme = S.cfg?.chat?.code_theme || "github-dark";
  return `<div class="code ${escAttr(theme)}"><div class="code-h"><span>${esc(lang || "code")}</span>
    <button data-cb="${id}" title="Копировать код"><svg><use href="#i-copy"/></svg></button></div>
    <pre class="${S.cfg?.chat?.code_scroll === false ? "noscroll" : ""}"><code id="${id}">${hl(body)}</code></pre></div>`;
}
function hl(src){
  return esc(src)
    .replace(/(\/\/.*?$)/gm,'<span class="c">$1</span>')
    .replace(/\b(function|return|const|let|var|type|interface|export|import|from|async|await|if|else|for|while|match|impl|struct|enum|pub|fn|use|mod)\b/g,'<span class="k">$1</span>')
    .replace(/("[^"]*"|'[^']*')/g,'<span class="s">$1</span>')
    .replace(/\b([A-Z][A-Za-z0-9_]*)\b/g,'<span class="t">$1</span>');
}
document.addEventListener("click", e => {
  const b = e.target.closest("[data-cb]");
  if (!b) return;
  const code = document.getElementById(b.dataset.cb);
  if (code) navigator.clipboard.writeText(code.textContent);
});

/* ── модель ─────────────────────────── */
function modelDisplay(raw){
  if (!raw) return "Assistant";
  return raw.length > 22 ? raw.slice(0, 19) + "…" : raw;
}

/* ── helpers ────────────────────────── */
function el(t,c){ const d = document.createElement(t); d.className = c; return d }
function shorten(s, n){
  s = String(s ?? "").replace(/\s+/g," ").trim();
  return s.length > n ? s.slice(0, n-1) + "…" : s;
}
function tail(s, n){
  s = String(s ?? "").replace(/\s+/g," ").trim();
  return s.length > n ? "…" + s.slice(-(n - 1)) : s;
}
function normalizeUiText(s){
  return String(s ?? "")
    .replace(/\s+/g," ")
    .replace(/\s+([,.!?;:])/g,"$1")
    .trim();
}
function renderEmptyIfNeeded(){
  const f = $("#feed");
  if (S.feed.length === 0){
    f.innerHTML = `<div class="empty">Продублируй вопрос текстом или ключевым словом — отвечу сразу.</div>`;
    return true;
  }
  return false;
}

/* ── рендер ленты ── */
function renderTranscriptGroup(item){
  const w = el("div","tg");
  const h = el("button","tg-h");
  h.textContent = `↳ Расшифровка аудио (${item.items.length})`;
  h.onclick = () => { item.open = !item.open; renderFeed(); };
  w.appendChild(h);
  if (item.open){
    const o = el("div","tg-open");
    item.items.forEach(r => {
      const row = el("div","row");
      if (r.live) row.classList.add("live");
      row.innerHTML = `<svg><use href="#i-mic-s"/></svg><span data-tid="${escAttr(r.id || "")}">${esc(r.text)}</span>`;
      o.appendChild(row);
    });
    w.appendChild(o);
  } else {
    const last = [...item.items].reverse().find(r => !r.live) || item.items[item.items.length-1];
    const c = el("div","tg-chip");
    c.innerHTML = `<svg><use href="#i-mic-s"/></svg><span data-tid="${escAttr(last ? last.id || "" : "")}">${esc(last ? last.text : "")}</span>`;
    w.appendChild(c);
  }
  return w;
}
function node(item){
  if (item.t === "user"){ const d = el("div","m-user"); d.textContent = item.text; return d; }
  if (item.t === "quick"){ const d = el("div","m-quick"); d.textContent = item.label; return d; }
  if (item.t === "tg") return renderTranscriptGroup(item);
  if (item.t === "ai"){
    const w = el("div","m-ai");
    w.innerHTML = `<div class="hd"><span class="av">✳</span><span class="name">${esc(item.model)}</span>
      <span class="spacer"></span>
      <button class="mini" data-a="copy"><svg><use href="#i-copy"/></svg></button>
      <button class="mini" data-a="spk"><svg><use href="#i-spk"/></svg></button>
      <button class="mini" data-a="del"><svg><use href="#i-del"/></svg></button></div>
      <div class="bd"></div>`;
    w.querySelector("[data-a=copy]").onclick = () => navigator.clipboard.writeText(item.buf);
    w.querySelector("[data-a=spk]").onclick  = () => invoke("tts_speak", { text: item.buf });
    w.querySelector("[data-a=del]").onclick  = () => { S.feed = S.feed.filter(x => x !== item); renderFeed(); };
    item.bd = w.querySelector(".bd"); item.bd.innerHTML = md(item.buf);
    return w;
  }
}
function renderFeed(){
  const f = $("#feed");
  f.innerHTML = "";
  if (renderEmptyIfNeeded()) return;
  S.feed.forEach(i => f.appendChild(node(i)));
  follow();
}
function scheduleRender(){
  if (rafPending) return;
  rafPending = true;
  requestAnimationFrame(() => { rafPending = false; renderFeed(); pinChipToEnd(); });
}
function follow(){
  const f = $("#feed");
  if (!S.autoscroll) return;
  const topMode = S.cfg?.chat?.order === "top";
  f.scrollTo({ top: topMode ? 0 : f.scrollHeight,
    behavior: (S.cfg?.chat.autoscroll_speed ?? 0) > 66 ? "smooth" : "auto" });
}
function pinChipToEnd(){
  const spans = document.querySelectorAll(".tg-chip span");
  const s = spans[spans.length - 1];
  if (s) s.scrollLeft = s.scrollWidth;
}
function patchRow(id, text){
  if (!id) return false;
  const s = document.querySelector(`[data-tid="${CSS.escape(String(id))}"]`);
  if (!s) return false;
  s.textContent = text;
  return true;
}

/* ── события пайплайна ── */
function ensureTg(){
  let tg = S.curTg;
  if (!tg){
    const collapse = S.cfg?.chat?.collapse_transcripts !== false;
    if (collapse){
      S.feed.forEach(i => { if (i.t === "tg") i.open = false; });
    }
    const collapseLast = S.cfg?.chat?.collapse_last !== false;
    tg = { t:"tg", items:[], open: !collapseLast };
    S.feed.push(tg); S.curTg = tg;
  }
  return tg;
}
function continuesWord(a, b){
  const ca = [...String(a)].pop(), cb = [...String(b)][0];
  return ca && cb && /[\p{L}\p{N}]/u.test(ca) && /\p{Ll}/u.test(cb);
}
function findItem(id){
  for (const it of S.feed)
    if (it.t === "tg") {
      const r = it.items.find(x => x.id === id);
      if (r) return r;
    }
  return null;
}
function finalizeLive(){
  for (const it of S.feed) if (it.t === "tg") {
    for (const r of it.items) if (r.live && r.text.trim()) r.live = false;
    it.items = it.items.filter(r => !(r.live && !r.text.trim()));
  }
}
event.listen("turn", e => {
  const id = e.payload.utt_id || e.payload.id;
  const text = normalizeUiText(e.payload.text);
  const r = findItem(id);
  if (r){ r.text = text; r.live = false; }
  else {
    const tg = S.curTg || ensureTg();
    const last = tg.items[tg.items.length - 1];
    if (last && !last.live && continuesWord(last.text, text)) last.text += text;
    else tg.items.push({ id, speaker: e.payload.speaker, text });
  }
  if (!patchRow(id, text)) renderFeed();
  pinChipToEnd();
});
event.listen("stt_partial", e => {
  const id = e.payload.utt_id;
  const text = normalizeUiText(e.payload.text);
  let r = findItem(id);
  if (!r){
    const tg = S.curTg || ensureTg();
    r = { id, speaker: "I", text: "", live: true };
    tg.items.push(r);
    renderFeed();
  } else {
    r.text = text;
    if (!patchRow(id, text)) scheduleRender();
  }
  follow(); pinChipToEnd();
});
event.listen("turn_update", e => {
  const { id, text } = e.payload;
  const cleaned = normalizeUiText(text);
  S.feed.forEach(i => {
    if (i.t === "tg") i.items.forEach(r => { if (r.id === id) r.text = cleaned; });
  });
  const elq = document.querySelector(`[data-tid="${CSS.escape(id)}"]`);
  if (elq) elq.textContent = cleaned;
});
event.listen("answer_token", e => {
  if (!S.curAi){
    S.curAi = { t:"ai", model: S.modelDisplay || "Assistant", buf:"" };
    S.feed.push(S.curAi); S.curTg = null;
    renderFeed();
  }
  S.curAi.buf += e.payload || "";
  requestAnimationFrame(() => { if (S.curAi?.bd) { S.curAi.bd.innerHTML = md(S.curAi.buf); follow(); } });
});
event.listen("answer_done", () => { S.curAi = null; $("#btnStop").hidden = true; });
event.listen("status", e => {
  const stop = $("#btnStop");
  if (e.payload === "generating"){ S.curTg = null; stop.hidden = false; }
  else stop.hidden = true;
});
$("#btnStop").onclick = () => invoke("cancel_generation").catch(console.error);
event.listen("error", e => {
  const b = $("#errBadge");
  if (!b) return;
  b.textContent = "!";
  b.title = String(e.payload || "Ошибка");
  b.style.display = "block";
  setTimeout(() => { b.style.display = "none"; }, 5000);
});

/* ── действия пользователя: логируем как quick ── */
function act(label, note, fn){
  finalizeLive();
  const compact = S.cfg?.chat?.compact_quick !== false;
  S.feed.push({ t:"quick", label: compact ? label : (note || label) });
  renderFeed(); fn();
}
$("#qaSay").onclick     = () => act("Что сказать", null, () => invoke("manual_trigger",{note:null}));
$("#qaSummary").onclick = () => act("Резюме", "сжато перескажи суть диалога", () => invoke("manual_trigger",{note:"сжато перескажи суть диалога"}));
$("#qaAnalyze").onclick = () => act("Разбор экрана", null,() => invoke("screen_analyze",{windowOnly:false}));
$("#btnSend").onclick   = () => { const v = $("#input").value.trim(); if (!v) return;
  finalizeLive();
  $("#input").value = ""; S.feed.push({ t:"user", text:v }); renderFeed(); invoke("manual_trigger",{note:v}); };
$("#input").addEventListener("keydown", e => { if (e.key === "Enter") $("#btnSend").click(); });

/* ── тумблеры/дропдауны (хелперы) ── */
function bindTgl(id, get, set){ const b = $(id);
  const up = () => b.classList.toggle("on", !!get());
  b.onclick = async () => { await set(!get()); up(); };
  b._up = up; return b; }
function bindDd(btnId, paneId){ const b = $(btnId), p = $(paneId);
  b.onclick = e => { e.stopPropagation(); document.querySelectorAll(".pane").forEach(x => x.hidden = true);
    const r = b.getBoundingClientRect();
    p.hidden = false;
    const ph = p.offsetHeight;
    p.style.left = Math.min(Math.max(8, r.left), window.innerWidth - p.offsetWidth - 8) + "px";
    if (r.bottom + 7 + ph <= window.innerHeight - 8){
      p.style.top = (r.bottom + 7) + "px";
      p.style.maxHeight = "";
    } else {
      p.style.top = Math.max(8, r.top - ph - 7) + "px";
      p.style.maxHeight = (r.top - 15) + "px";
    }
    p.hidden = false; };
  document.addEventListener("click", e => { if (!p.hidden && !p.contains(e.target)) p.hidden = true; }); }

/* ── панели ── */
$("#btnMute").onclick = async () => {
  const m = $("#btnMute"); const muted = m.classList.toggle("on");
  m.style.color = muted ? "var(--err)" : "";
  await invoke("mic_mute", { muted });
};
$("#btnShotRegion").onclick = () => act("Скриншот области", null, () => invoke("screen_analyze",{windowOnly:true}));
$("#btnShotFull").onclick  = () => act("Скриншот экрана", null,  () => invoke("screen_analyze",{windowOnly:false}));
$("#btnReset").onclick = () => { S.feed = []; S.curAi = S.curTg = null; renderFeed(); invoke("ctx_reset"); };
$("#btnHome").onclick = () => invoke("go_home");
const feedEl = $("#feed");
const fabDown = $("#fabDown");
feedEl.addEventListener("scroll", () => {
  const atBottom = feedEl.scrollHeight - feedEl.scrollTop - feedEl.clientHeight < 24;
  S.autoscroll = atBottom;
  fabDown.hidden = atBottom;
});
fabDown.onclick = () => { S.autoscroll = true; follow(); };
fabDown.hidden = true;

/* --- pane: STT --- */
async function fillSttPane(){
  try {
    const s = await invoke("stt_get");
    $("#selSttProvider").value = s.provider;
    $("#selSttModel").value = s.model;
    $("#selSttMode").value = s.mode;
    $("#selSttLang").value = s.language;
  } catch (e) { console.error(e); }
}
function sttSave(){
  const provider = $("#selSttProvider").value;
  let model = $("#selSttModel").value;
  if (!model || ![...$("#selSttModel").options].some(o => o.value === model)) {
    model = provider === "soniox" ? "stt-rt-v5" : provider === "deepgram" ? "nova-3-general" : "whisper-large-v3-turbo";
    $("#selSttModel").value = model;
  }
  invoke("stt_set",{ provider, model, mode: $("#selSttMode").value, language: $("#selSttLang").value }).catch(console.error);
}
$("#selSttProvider").onchange = () => {
  const m = $("#selSttProvider").value === "soniox" ? "stt-rt-v5" : $("#selSttProvider").value === "deepgram" ? "nova-3-general" : "whisper-large-v3-turbo";
  $("#selSttModel").value = m;
  sttSave();
};
$("#selSttModel").onchange = sttSave;
$("#selSttMode").onchange = sttSave;
$("#selSttLang").onchange = sttSave;

/* --- pane: заметки --- */
$("#btnNotes").onclick = e => { e.stopPropagation();
  const r = $("#btnNotes").getBoundingClientRect(); const p = $("#paneNotes");
  p.style.top = (r.bottom + 6) + "px"; p.style.left = r.left + "px";
  document.querySelectorAll(".pane").forEach(x => { if (x !== p) x.hidden = true; });
  invoke("notes_list").then(list => {
    const c = $("#notesList"); c.innerHTML = "";
    if (!list.length){
      const d = el("div","item empty-item"); d.textContent = "Заметок пока нет";
      c.appendChild(d);
    }
    list.forEach(n => { const d = el("button","item"); d.textContent = n.name;
      d.onclick = async () => { const full = await invoke("note_get",{ id: n.id });
        $("#noteText").textContent = full.text; $("#notePane").hidden = false; };
      c.appendChild(d); });
    if (list.length > 1){
      const all = el("button","item"); all.textContent = "Показать все";
      all.onclick = async () => { const list2 = await invoke("notes_list");
        const texts = (await Promise.all(list2.map(n => invoke("note_get",{ id: n.id })))).map(n => n.text).join("\n\n");
        $("#noteText").textContent = texts; $("#notePane").hidden = false; };
      c.appendChild(all);
    }
  }).catch(console.error);
  p.hidden = !p.hidden;
};
$("#notePane").onclick = () => { $("#notePane").hidden = true; };

/* --- pane: контекст --- */
$("#ddContext").onclick = async e => { e.stopPropagation();
  const r = $("#ddContext").getBoundingClientRect(); const p = $("#paneCtx");
  p.style.top = (r.bottom + 6) + "px"; p.style.left = r.left + "px";
  document.querySelectorAll(".pane").forEach(x => { if (x !== p) x.hidden = true; });
  const [list, cur] = await Promise.all([
    invoke("contexts_list").catch(() => []),
    invoke("context_current").catch(() => ""),
  ]);
  const c = $("#ctxList"); c.innerHTML = "";
  const mk = (id, name) => { const d = el("button","item" + (id === cur ? " on" : ""));
    d.textContent = name;
    d.onclick = async () => {
      try {
        await invoke("context_apply", { id });
        $("#ctxName").textContent = id ? name : "Контекст";
        toast(id ? "Контекст: " + name : "Контекст отключён");
        p.hidden = true;
      } catch (err) { toast(String(err), true); }
    };
    c.appendChild(d);
  };
  mk("", "— без контекста —");
  list.forEach(ctx => mk(ctx.id, ctx.name));
  if (!list.length){
    const d = el("div","item empty-item"); d.textContent = "Контекстов нет (создайте в настройках)";
    c.appendChild(d);
  }
  p.hidden = !p.hidden;
};

/* --- pane: функции ИИ --- */
$("#tglSearch").onchange   = e => invoke("search_set",{ on: e.target.checked }).catch(console.error);
$("#tglNotesRag").onchange = e => invoke("notes_rag_set",{ on: e.target.checked }).catch(console.error);

/* --- pane: выбор модели ── */
const EFFORTS = [null, "minimal", "low", "medium", "high"];
const EFFORT_LABELS = ["Выключен", "Минимальный", "Низкий", "Средний", "Максимальный"];
// models_list теперь отдаёт объекты ModelMetadata:
// {id, name, family, context_length, pricing:{input_per_1m,output_per_1m}, capabilities}
let mmAll = [], mmSelected = "", mmFam = "__all", mmGroups = {};

function effortIndex(e){ const i = EFFORTS.indexOf(e ?? null); return i < 0 ? 0 : i; }
function updateReasonLabel(){
  $("#mmReasonVal").textContent = EFFORT_LABELS[Number($("#mmRange").value)];
}
function fmtCtx(n){
  if (!n) return "";
  return n >= 1000 ? Math.round(n / 1000) + "k" : String(n);
}
function fmtPrice(p){
  if (!p || p <= 0) return "";
  const v = p >= 1 ? p.toFixed(2) : p >= 0.01 ? p.toFixed(2) : p.toFixed(3);
  return "$" + parseFloat(v);
}
function metaBadges(m){
  const out = [];
  const ctx = fmtCtx(m.context_length);
  if (ctx) out.push('<span class="mm-b">' + ctx + "</span>");
  const pi = fmtPrice(m.pricing && m.pricing.input_per_1m);
  const po = fmtPrice(m.pricing && m.pricing.output_per_1m);
  if (pi || po) out.push('<span class="mm-b">' + esc((pi || "?") + "/" + (po || "?")) + "</span>");
  if (m.capabilities && m.capabilities.vision) out.push('<span class="mm-b">vision</span>');
  if (m.capabilities && m.capabilities.tools) out.push('<span class="mm-b">tools</span>');
  if (m.capabilities && m.capabilities.reasoning) out.push('<span class="mm-b">reasoning</span>');
  return out.join("");
}
function renderProviders(){
  const p = $("#mmProviders"); p.innerHTML = "";
  // Только семейства из метаданных models_list — без «хоста» и без __all.
  const fams = Object.keys(mmGroups).sort();
  if (!fams.length) return;
  if (!mmGroups[mmFam]) mmFam = fams[0];
  fams.forEach((f, i) => {
    const b = el("button", "mm-prov" + (mmFam === f ? " on" : ""));
    b.innerHTML = '<span class="dot dot-f' + (i % 8) + '"></span><span>' + esc(f) + "</span>";
    b.onclick = () => { mmFam = f; renderProviders(); renderModels(); };
    p.appendChild(b);
  });
}
function renderModels(){
  const list = $("#mmModels"); list.innerHTML = "";
  const arr = mmFam === "__all"
    ? [...mmAll].sort((a, b) => a.id.localeCompare(b.id))
    : (mmGroups[mmFam] || []).sort((a, b) => a.id.localeCompare(b.id));
  if (!arr.length){ list.innerHTML = '<div class="empty-note">Нет моделей</div>'; return; }
  arr.forEach(m => {
    const b = el("button", "mm-model" + (m.id === mmSelected ? " on" : ""));
    b.innerHTML = '<div class="info"><div class="name">' + esc(m.name || m.id) +
      '</div><div class="id">' + esc(m.id) + '</div><div class="badges">' +
      metaBadges(m) + '</div></div>' +
      (m.id === mmSelected ? '<span class="chk">✓</span>' : "");
    b.onclick = async () => {
      const eff = EFFORTS[Number($("#mmRange").value)] ?? null;
      try {
        await invoke("llm_set", { model: m.id, effort: eff });
        mmSelected = m.id;
        $("#modelName").textContent = modelDisplay(m.name || m.id);
        renderModels();
      } catch (e) { toast(String(e), true); }
    };
    list.appendChild(b);
  });
}
async function openModelModal(){
  $("#modelModal").hidden = false;
  const prov = $("#mmProviders"), list = $("#mmModels");
  prov.innerHTML = ""; list.innerHTML = '<div class="empty-note">Загрузка…</div>';
  let cfg = {};
  try {
    [mmAll, cfg] = await Promise.all([invoke("models_list"), invoke("get_config")]);
  } catch (e) {
    list.innerHTML = '<div class="empty-note">Ошибка: ' + esc(String(e)) + "</div>";
    return;
  }
  if (!Array.isArray(mmAll)) mmAll = [];
  mmSelected = (cfg.llm && cfg.llm.model) || "";
  mmGroups = {};
  mmAll.forEach(m => { (mmGroups[m.family] ||= []).push(m); });
  mmFam = mmSelectedFamily();
  renderProviders();
  renderModels();
  $("#mmRange").value = effortIndex(cfg.llm ? cfg.llm.reasoning_effort : null);
  updateReasonLabel();
}
function mmSelectedFamily(){
  const f = mmAll.find(m => m.id === mmSelected)?.family;
  return (f && mmGroups[f]) ? f : Object.keys(mmGroups).sort()[0] || "__all";
}
$("#ddModel").onclick = e => { e.stopPropagation(); openModelModal(); };
$("#mmClose").onclick = () => { $("#modelModal").hidden = true; };
$("#modelModal").onclick = e => { if (e.target.id === "modelModal") $("#modelModal").hidden = true; };
$("#mmRange").oninput = updateReasonLabel;
$("#mmRange").onchange = async () => {
  const eff = EFFORTS[Number($("#mmRange").value)] ?? null;
  try { await invoke("llm_set", { model: mmSelected, effort: eff }); }
  catch (e) { toast(String(e), true); }
};

/* ── init ── */
(async () => {
  S.cfg = await invoke("ui_get").catch(() => ({}));
  const ui = S.cfg.ui || {};
  const chat = S.cfg.chat || {};
  const root = document.documentElement;

  const accent = ui.accent || "#f97316";
  root.style.setProperty("--accent", accent);
  const hex = /^#?([0-9a-f]{6})$/i.exec(String(accent).trim());
  if (hex){
    const n = parseInt(hex[1], 16);
    root.style.setProperty("--accent-rgb", `${(n>>16)&255},${(n>>8)&255},${n&255}`);
  } else if (ui.accent_rgb){
    root.style.setProperty("--accent-rgb", ui.accent_rgb);
  }

  const opacity = Number(ui.opacity ?? 84);
  root.style.setProperty("--surface", String(Math.max(10, Math.min(100, opacity)) / 100));

  const fs = Number(chat.font_size ?? 13.5);
  root.style.setProperty("--fs", fs + "px");

  const rawModel = S.cfg.llm_model || S.cfg.model || "";
  S.modelDisplay = S.cfg.model_display || modelDisplay(rawModel);
  $("#modelName").textContent = S.modelDisplay;

  bindTgl("#btnAuto", () => S.auto,  v => { S.auto = v; return invoke("auto_answers_set",{on:v}); })._up();
  bindTgl("#btnTts",  () => S.tts,   v => { S.tts = v;  return invoke("tts_auto_set",{on:v}); })._up();
  bindTgl("#btnChats",() => S.rail,  v => { $("#rail").classList.toggle("hidden", !v); return invoke("ui_set",{key:"rail",value:v}); });
  bindDd("#btnStt","#paneStt");
  bindDd("#btnAi","#paneAi");
  S.auto = (await invoke("auto_answers_get").catch(() => false));
  S.tts = (await invoke("tts_auto_get").catch(() => false));
  S.rail = ui.rail !== false;
  S.autoscroll = chat.autoscroll !== false;
  $("#feed").classList.toggle("top", chat.order === "top");
  $("#btnAuto")._up(); $("#btnTts")._up(); $("#btnChats")._up();
  fillSttPane();
  refreshChats();
})();
event.listen("meeting", async () => {
  if (S.chat) S.feeds[S.chat] = S.feed;
  S.chat = null; S.feeds = {}; S.loaded = {}; S.feed = []; S.curAi = S.curTg = null;
  await refreshChats();
  if (S.chats.length) {
    await invoke("chat_switch",{ id: S.chats[0].id });
    setChat(S.chats[0].id);
  } else renderChats();
});
async function setChat(id){
  if (S.chat && S.chat !== id) S.feeds[S.chat] = S.feed;
  S.chat = id;
  S.curAi = S.curTg = null;
  if (id && S.feeds[id]) {
    S.feed = S.feeds[id];
  } else {
    S.feed = [];
    if (id) S.feeds[id] = S.feed;
  }
  renderChats(); renderFeed();
  if (id && !S.loaded[id]) {
    S.loaded[id] = true;
    try {
      const msgs = await invoke("chat_messages", { id });
      if (msgs && msgs.length) {
        const items = [];
        let tg = null;
        for (const m of msgs) {
          if (m.speaker === "I") {
            if (!tg) { tg = { t:"tg", items:[], open:false }; items.push(tg); }
            tg.items.push({ speaker: "I", text: m.text });
          } else {
            tg = null;
            items.push(m.speaker === "user"
              ? { t:"user", text:m.text }
              : { t:"ai", model:S.modelDisplay || "assistant", buf:m.text });
          }
        }
        S.feed = items;
        S.feeds[id] = S.feed;
        renderFeed();
      }
    } catch (e) { console.error(e); }
  }
}
async function refreshChats(){
  try { S.chats = await invoke("chats_list"); } catch (e) { S.chats = []; }
  renderChats();
}
function renderChats(){ const c = $("#chats"); c.innerHTML = "";
  S.chats.forEach(ch => { const d = el("button","chat-n" + (ch.id === S.chat ? " on":""));
    d.textContent = ch.number; d.onclick = async () => { await invoke("chat_switch",{id:ch.id}); setChat(ch.id); };
    c.appendChild(d); }); }
$("#btnNewChat").onclick = async () => { const id = await invoke("chat_create");
  await invoke("chat_switch",{id});
  await refreshChats();
  setChat(id);
};

/* ── перемещение и растягивание окна ── */
const tauriWin = () => window.__TAURI__.window.getCurrentWindow();
function cornerResize(sel, dir){
  const el = $(sel);
  el.addEventListener("mousedown", e => {
    e.preventDefault();
    tauriWin().startResizeDragging(dir);
  });
}
cornerResize(".corner.nw","NorthWest");
cornerResize(".corner.ne","NorthEast");
cornerResize(".corner.sw","SouthWest");
cornerResize(".corner.se","SouthEast");
