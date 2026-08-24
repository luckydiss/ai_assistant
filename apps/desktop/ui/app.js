const { core } = window.__TAURI__;

const navLinks = document.querySelectorAll("nav a");
const meetingsView = document.getElementById("meetings-view");
const contextsView = document.getElementById("contexts-view");
const settingsView = document.getElementById("settings-view");

function esc(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
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

function route() {
  const hash = location.hash.replace("#", "") || "meetings";
  const active = ["meetings", "contexts", "settings"].includes(hash) ? hash : "meetings";
  navLinks.forEach(a =>
    a.classList.toggle("active", a.dataset.view === active)
  );
  meetingsView.classList.toggle("hidden", active !== "meetings");
  contextsView.classList.toggle("hidden", active !== "contexts");
  settingsView.classList.toggle("hidden", active !== "settings");
  if (active === "meetings") renderMeetings();
  else if (active === "contexts") renderContexts();
  else renderSettings();
}

// ---------- meetings ----------
let meetings = [];

async function renderMeetings() {
  try {
    meetings = await core.invoke("meetings_list");
  } catch (e) {
    meetings = [];
  }
  const contexts = await core.invoke("contexts_list").catch(() => []);

  const form = document.createElement("div");
  form.className = "card";
  form.innerHTML = '<h2>Новая встреча</h2>' +
    '<div class="row"><input id="m-name" placeholder="Название">' +
    '<input id="m-vacancy" placeholder="Вакансия"></div>' +
    '<button id="m-create" class="primary">Создать</button>';
  meetingsView.innerHTML = "";
  meetingsView.appendChild(form);

  form.querySelector("#m-create").onclick = async () => {
    const name = form.querySelector("#m-name").value.trim();
    const vacancy = form.querySelector("#m-vacancy").value.trim();
    if (!name) return;
    const id = await core.invoke("meeting_create", { name, vacancy });
    await renderMeetings();
    startMeeting(id);
  };

  for (const m of meetings) {
    const card = document.createElement("div");
    card.className = "card";
    const ctxSelect = '<select data-mid="' + m.id + '">' +
      '<option value="">— без контекста —</option>' +
      contexts.map(c =>
        '<option value="' + c.id + '"' + (m.context_id === c.id ? " selected" : "") + ">" +
        esc(c.name) + "</option>"
      ).join("") + "</select>";
    card.innerHTML =
      '<div class="title">' + esc(m.name) + "</div>" +
      '<div class="sub">' + fmtDate(m.created_at) +
      (m.vacancy ? " · " + esc(m.vacancy) : "") +
      " · сообщений: " + m.messages + "</div>" +
      '<div class="row">' + ctxSelect +
      '<button data-continue="' + m.id + '" class="primary">Продолжить</button>' +
      '<button data-rename="' + m.id + '">Переименовать</button>' +
      '<button data-delete="' + m.id + '">Удалить</button></div>';
    meetingsView.appendChild(card);

    card.querySelector("[data-continue]").onclick = () => startMeeting(m.id);
    card.querySelector("[data-delete]").onclick = async () => {
      await core.invoke("meeting_delete", { id: m.id });
      await renderMeetings();
    };
    card.querySelector("[data-rename]").onclick = async () => {
      const name = prompt("Новое имя:", m.name);
      if (name && name.trim()) {
        await core.invoke("meeting_rename", { id: m.id, name: name.trim() });
        await renderMeetings();
      }
    };
    card.querySelector("select").onchange = async (e) => {
      await core.invoke("meeting_set_context", {
        meetingId: m.id,
        contextId: e.target.value,
      });
    };
  }
}

async function startMeeting(id) {
  await core.invoke("start_pipeline", { meetingId: id });
}

// ---------- contexts ----------
async function renderContexts() {
  const contexts = await core.invoke("contexts_list").catch(() => []);
  contextsView.innerHTML = "";

  const form = document.createElement("div");
  form.className = "card";
  form.innerHTML = '<h2>Новый контекст</h2>' +
    '<label>Имя</label><input id="c-name">' +
    '<label>Роль</label><input id="c-role">' +
    '<label>Языки (через запятую)</label><input id="c-langs" value="ru, en">' +
    '<label>Резюме (TXT/MD)</label><textarea id="c-resume" rows="5"></textarea>' +
    '<div class="row"><button id="c-import">Загрузить файл</button>' +
    '<input type="file" id="c-file" accept=".txt,.md" class="hidden"></div>' +
    '<label>Extra-промпт</label><textarea id="c-extra" rows="2"></textarea>' +
    '<button id="c-save" class="primary">Сохранить</button>';
  contextsView.appendChild(form);

  form.querySelector("#c-import").onclick = () => form.querySelector("#c-file").click();
  form.querySelector("#c-file").onchange = async (e) => {
    const f = e.target.files[0];
    if (!f) return;
    const text = await f.text();
    form.querySelector("#c-resume").value = text;
  };
  form.querySelector("#c-save").onclick = async () => {
    const ctx = {
      id: crypto.randomUUID(),
      name: form.querySelector("#c-name").value.trim(),
      role: form.querySelector("#c-role").value.trim(),
      languages: form.querySelector("#c-langs").value.split(",").map(s => s.trim()).filter(Boolean),
      resumeText: form.querySelector("#c-resume").value,
      extraPrompt: form.querySelector("#c-extra").value,
    };
    if (!ctx.name) return;
    await core.invoke("context_save", { ctx });
    await renderContexts();
  };

  for (const c of contexts) {
    const card = document.createElement("div");
    card.className = "card";
    card.innerHTML =
      '<div class="title">' + esc(c.name) + "</div>" +
      '<div class="sub">' + (c.role ? esc(c.role) : "—") + "</div>" +
      '<button data-edit="' + c.id + '">Редактировать</button> ' +
      '<button data-del="' + c.id + '">Удалить</button>';
    contextsView.appendChild(card);

    card.querySelector("[data-del]").onclick = async () => {
      await core.invoke("context_delete", { id: c.id });
      await renderContexts();
    };
    card.querySelector("[data-edit]").onclick = () => {
      form.querySelector("#c-name").value = c.name;
      form.querySelector("#c-role").value = c.role;
      form.querySelector("#c-langs").value = c.languages.join(", ");
      form.querySelector("#c-resume").value = c.resumeText;
      form.querySelector("#c-extra").value = c.extraPrompt;
      form.querySelector("#c-save").onclick = async () => {
        const updated = {
          id: c.id,
          name: form.querySelector("#c-name").value.trim(),
          role: form.querySelector("#c-role").value.trim(),
          languages: form.querySelector("#c-langs").value.split(",").map(s => s.trim()).filter(Boolean),
          resumeText: form.querySelector("#c-resume").value,
          extraPrompt: form.querySelector("#c-extra").value,
        };
        await core.invoke("context_save", { ctx: updated });
        await renderContexts();
      };
      location.hash = "#contexts";
    };
  }
}

const HOTKEY_ACTIONS = [
  ["manual", "Что сказать"],
  ["hide", "Скрыть оверлей"],
  ["click_through", "Click-through"],
  ["mute", "Mute"],
  ["record", "Запись"],
  ["screenshot_full", "Скриншот (весь)"],
  ["screenshot_region", "Скриншот (регион)"],
];

async function renderSettings() {
  settingsView.innerHTML = "";

  const hkCard = document.createElement("div");
  hkCard.className = "card";
  hkCard.innerHTML = "<h2>Горячие клавиши</h2>";
  let hk;
  try {
    hk = await core.invoke("hotkeys_get");
  } catch {
    hk = null;
  }
  for (const [action, label] of HOTKEY_ACTIONS) {
    const val = hk ? hk[action] : "";
    const row = document.createElement("div");
    row.className = "row";
    row.innerHTML = "<span style='width:180px'>" + esc(label) + "</span>" +
      '<input data-accel="' + action + '" value="' + esc(val) + '" placeholder="(пусто = отключено)">' +
      '<button data-save-hk="' + action + '">Сохранить</button>';
    hkCard.appendChild(row);
  }
  settingsView.appendChild(hkCard);

  hkCard.querySelectorAll("[data-save-hk]").forEach(btn => {
    btn.onclick = async () => {
      const action = btn.dataset.saveHk;
      const accel = hkCard.querySelector('[data-accel="' + action + '"]').value.trim();
      try {
        await core.invoke("set_hotkey", { action, accel });
        btn.textContent = "OK";
        setTimeout(() => (btn.textContent = "Сохранить"), 1200);
      } catch (e) {
        btn.textContent = "Ошибка: " + e;
      }
    };
  });

  const audioCard = document.createElement("div");
  audioCard.className = "card";
  audioCard.innerHTML = "<h2>Запись</h2>";
  settingsView.appendChild(audioCard);

  let devices = [];
  try {
    devices = await core.invoke("list_audio_devices");
  } catch {
    devices = [];
  }
  audioCard.innerHTML +=
    '<label>Источник</label>' +
    '<div class="row"><select id="s-source">' +
    '<option value="system+mic">system+mic</option>' +
    '<option value="system">system</option>' +
    '<option value="mic">mic</option></select></div>' +
    '<label>Режим</label>' +
    '<div class="row"><select id="s-mode">' +
    '<option value="manual">manual</option>' +
    '<option value="vad">vad</option></select></div>' +
    '<label>Микрофон</label>' +
    '<div class="row"><select id="s-mic">' +
    '<option value="">по умолчанию</option>' +
    devices.map(d => '<option value="' + esc(d) + '">' + esc(d) + "</option>").join("") +
    "</select></div>" +
    '<div class="row"><button id="s-save" class="primary">Сохранить</button></div>';

  try {
    const cfg = await core.invoke("get_config");
    audioCard.querySelector("#s-source").value = cfg.audio.source || "system+mic";
    audioCard.querySelector("#s-mode").value = cfg.audio.mode || "manual";
    if (cfg.audio.micDevice) {
      audioCard.querySelector("#s-mic").value = cfg.audio.micDevice;
    }
  } catch {
    // get_config может отсутствовать — оставляем дефолты
  }

  audioCard.querySelector("#s-save").onclick = async () => {
    try {
      await core.invoke("update_audio_settings", {
        source: audioCard.querySelector("#s-source").value,
        mode: audioCard.querySelector("#s-mode").value,
        micDevice: audioCard.querySelector("#s-mic").value || null,
      });
    } catch (e) {
      alert("Ошибка: " + e);
    }
  };

  const ttsCard = document.createElement("div");
  ttsCard.className = "card";
  ttsCard.innerHTML = "<h2>Озвучка ответов (Cartesia)</h2>";
  settingsView.appendChild(ttsCard);
  ttsCard.innerHTML +=
    '<label>Режим озвучки</label>' +
    '<div class="row"><select id="s-tts-mode">' +
    '<option value="off">Выкл</option>' +
    '<option value="auto">Авто (стриминг)</option>' +
    '<option value="hotkey">По хоткею (Ctrl+T)</option>' +
    "</select>" +
    '<button id="s-tts-save" class="primary">Сохранить</button></div>' +
    '<p class="hint">Ключ API задаётся в config.toml ([tts] api_key)</p>';

  try {
    const cfg = await core.invoke("get_config");
    ttsCard.querySelector("#s-tts-mode").value = (cfg.tts && cfg.tts.mode) || "off";
  } catch {
    // get_config может отсутствовать — оставляем off
  }

  ttsCard.querySelector("#s-tts-save").onclick = async () => {
    try {
      await core.invoke("tts_set_mode", {
        mode: ttsCard.querySelector("#s-tts-mode").value,
      });
      const btn = ttsCard.querySelector("#s-tts-save");
      btn.textContent = "OK";
      setTimeout(() => (btn.textContent = "Сохранить"), 1200);
    } catch (e) {
      alert("Ошибка: " + e);
    }
  };

  const protCard = document.createElement("div");
  protCard.className = "card";
  protCard.innerHTML = "<h2>Защита от записи экрана</h2>";
  settingsView.appendChild(protCard);

  let protection = false;
  try {
    const cfg = await core.invoke("get_config");
    protection = !!(cfg.ui && cfg.ui.protection);
  } catch {}
  protCard.innerHTML +=
    '<label>Режим защиты (окно не видно в демонстрации/записи экрана)</label>' +
    '<div class="row"><button id="s-prot" class="' + (protection ? "primary" : "") + '">' +
    (protection ? "Включён" : "Выключен") + "</button></div>" +
    '<p class="hint">По умолчанию выключен</p>';

  protCard.querySelector("#s-prot").onclick = async () => {
    const target = !protection;
    try {
      await core.invoke("protection_set", { on: target });
      protection = target;
      const b = protCard.querySelector("#s-prot");
      b.className = protection ? "primary" : "";
      b.textContent = protection ? "Включён" : "Выключен";
    } catch (e) {
      alert("Ошибка: " + e);
    }
  };
}

window.addEventListener("hashchange", route);
route();