# Design: Overlay UI

## 1. tauri.conf.json (заменить окно)

```json
{
  "productName": "Interview Assistant",
  "version": "0.1.0",
  "identifier": "com.interview.assistant",
  "build": { "frontendDist": "../ui", "withGlobalTauri": true },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "Interview Assistant",
        "width": 480,
        "height": 640,
        "transparent": true,
        "decorations": false,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "resizable": false,
        "shadow": false
      }
    ],
    "security": { "csp": null }
  }
}
```

## 2. apps/desktop/ui/index.html

```html
<!doctype html>
<html>
<head>
<meta charset="utf-8">
<style>
  html, body { margin: 0; background: transparent; overflow: hidden; }
  #card {
    margin: 12px; padding: 14px; border-radius: 12px;
    background: rgba(20,20,24,0.92); color: #e8e8ec;
    font: 13px/1.5 system-ui, sans-serif;
    max-height: 600px; overflow-y: auto;
    -webkit-user-select: text;
  }
  #status { font-size: 11px; opacity: .7; margin-bottom: 6px; }
  #status.gen { color: #ffb454; }
  pre { background: #111; padding: 8px; border-radius: 6px; overflow-x: auto; }
  code { font-family: ui-monospace, Consolas, monospace; }
  li { margin: 2px 0; }
  .hidden { display: none; }
</style>
</head>
<body>
  <div id="card" class="hidden">
    <div id="status">listening</div>
    <div id="answer"></div>
  </div>
  <script src="app.js"></script>
</body>
</html>
```

## 3. apps/desktop/ui/app.js

```js
const { event } = window.__TAURI__;
const card = document.getElementById("card");
const statusEl = document.getElementById("status");
const answerEl = document.getElementById("answer");
let buf = "";

function esc(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function render(md) {
  let html = "";
  const parts = md.split("```");
  for (let i = 0; i < parts.length; i++) {
    if (i % 2 === 1) {
      const body = parts[i].replace(/^[a-z]*\n/, "");
      html += "<pre><code>" + esc(body) + "</code></pre>";
    } else {
      let t = esc(parts[i]);
      t = t.replace(/\*\*(.+?)\*\*/g, "<b>$1</b>");
      t = t.split("\n").map(line =>
        line.trim().startsWith("- ") ? "<li>" + line.trim().slice(2) + "</li>" : line
      ).join("\n");
      html += t.replace(/\n/g, "<br>");
    }
  }
  answerEl.innerHTML = html;
}

event.listen("answer_token", e => {
  card.classList.remove("hidden");
  buf += e.payload;
  requestAnimationFrame(() => render(buf));
});
event.listen("answer_done", () => { statusEl.textContent = "listening"; statusEl.className = ""; });
event.listen("answer_skipped", () => { card.classList.add("hidden"); buf = ""; });
event.listen("status", e => {
  statusEl.textContent = e.payload;
  statusEl.className = e.payload === "generating" ? "gen" : "";
  if (e.payload === "generating") { buf = ""; answerEl.innerHTML = ""; card.classList.remove("hidden"); }
});
event.listen("turn", e => {
  console.log("turn:", e.payload.speaker, e.payload.text);
});
```

## Рассмотрено и отклонено
- **React+Vite:** отклонено для MVP — нет npm-сборки, меньше точек отказа у слабого агента
- **marked.js с CDN:** отклонено — нет интернета; собственный markdown-subset за 30 строк
