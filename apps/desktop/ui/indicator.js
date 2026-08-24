const { event, core } = window.__TAURI__;

const bProt = document.getElementById("bProt");
const bRec = document.getElementById("bRec");
const bAuto = document.getElementById("bAuto");
const bTts = document.getElementById("bTts");

function apply(p) {
  bProt.textContent = p.protection ? "Защита" : "Защита ОТКЛ";
  bProt.className = "b " + (p.protection ? "on" : "off");
  bRec.className = "b " + (p.recording ? "on" : "");
  bAuto.className = "b " + (p.auto ? "on" : "");
  bTts.className = "b " + (p.tts !== "off" ? "on" : "");
}

event.listen("indicator", e => apply(e.payload));
core.invoke("indicator_get").then(apply).catch(() => {});