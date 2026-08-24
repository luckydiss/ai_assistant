# Design: Translations

## 1. engine-llm: complete + translate

```rust
impl LlmClient {
    pub async fn complete(&self, messages: Vec<ChatMessage>, max_tokens: u32, temperature: f32) -> Result<String, SttError> {
        let body = serde_json::json!({ "model": self.model, "messages": messages,
            "temperature": temperature, "max_tokens": max_tokens, "stream": false });
        let resp = self.http.post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key).json(&body).send().await?;
        if resp.status().as_u16() == 401 { return Err(SttError::Authentication); }
        if !resp.status().is_success() {
            return Err(SttError::Api { status: resp.status().as_u16(), message: "complete".into() });
        }
        let v: serde_json::Value = resp.json().await?;
        Ok(v["choices"][0]["message"]["content"].as_str().unwrap_or_default().to_string())
    }

    pub async fn translate(&self, text: &str, lang: &str) -> Result<String, SttError> {
        let msgs = vec![
            ChatMessage { role: Role::System, content: format!(
                "Переведи ответ собеседования на язык: {}. Сохрани markdown и код-блоки; \
                 код, команды и идентификаторы не переводи. Выведи только перевод.", lang) },
            ChatMessage { role: Role::User, content: text.to_string() },
        ];
        self.complete(msgs, 1000, 0.2).await
    }
}
```

## 2. Orchestrator

```rust
pub enum OrchEvent { ..., Translation { lang: String, text: String } }

#[derive(Clone, Default)]
struct TransHandles(Arc<Mutex<Vec<AbortHandle>>>);

// Inner: languages: Vec<String>, trans: TransHandles
pub fn set_languages(&mut self /*через Cmd::SetLangs*/, langs: Vec<String>)

// в fire(): self.trans.0.lock().unwrap().drain(..).for_each(|h| h.abort());

// в форвардере (ему передаются llm, languages, events, trans):
AnswerEvent::Done(full) => {
    let _ = events.send(OrchEvent::Done);
    for lang in languages.iter().skip(1).take(2).cloned().collect::<Vec<_>>() {
        let (llm, events, trans, full) = (llm.clone(), events.clone(), trans.clone(), full.clone());
        let h = tokio::spawn(async move {
            match llm.translate(&full, &lang).await {
                Ok(t) => { let _ = events.send(OrchEvent::Translation { lang, text: t }); }
                Err(e) => { let _ = events.send(OrchEvent::Error(e.to_string())); }
            }
        });
        trans.0.lock().unwrap().push(h.abort_handle());
    }
}
```

Cmd::SetLangs(Vec<String>) + публичный `set_languages(&self, v)` → try_send. pipeline.rs при старте: `orch.set_languages(ws_ctx.languages.clone())`.

PromptContext: добавить `pub languages: Vec<String>`; with_workspace сохраняет; commands.rs читает активный контекст → languages.

## 3. overlay.js

```js
event.listen("translation", e => {
  const div = document.createElement("div");
  div.className = "tr";
  div.innerHTML = "<b>" + e.payload.lang + "</b><br>" + render(e.payload.text);
  feed.appendChild(div); feed.scrollTop = feed.scrollHeight;
});
// CSS: .tr { opacity:.85; border-left:2px solid #555; padding-left:8px; margin-top:8px; }
```

## Рассмотрено и отклонено
- **Отдельная translation-модель в конфиге:** отклонено — тот же endpoint/model, temperature 0.2
- **Перевод стрима:** отклонено
