use engine_context::ContextBuilder;
use engine_dialogue::{Speaker, Turn};
use engine_llm::{AnswerEvent, LlmClient};
use std::time::Duration;
use tokio::time::timeout;

const API_KEY: &str =
    "sk-rt_be0957c9790e07d7af0dc0c56e5aa7999e3025c21c65ec5cdfa81ab646a0fa26932414";
const BASE: &str = "https://api.dslab.tech/v1";
const MODEL: &str = "gemini-2.5-flash-lite";

const SYSTEM: &str = "Ты — невидимый ассистент на техническом собеседовании.
Даётся диалог: I — интервьюер, C — кандидат.
Кандидат сам запрашивает подсказку, когда нужна помощь.
Отвечай сразу суть: 2–5 буллетов или короткий код-блок.
Язык = язык последнего вопроса I или языка запроса.
Без вступлений и мета-комментариев.";

fn turns() -> Vec<Turn> {
    let now = chrono::Utc::now();
    vec![
        Turn {
            speaker: Speaker::Interviewer,
            text: "просите посчитать буквы в слове Moje".into(),
            start_time: now,
            end_time: now,
        },
        Turn {
            speaker: Speaker::Candidate,
            text: "просите посчитать буквы".into(),
            start_time: now,
            end_time: now,
        },
        Turn {
            speaker: Speaker::Candidate,
            text: "мне нужна помощь".into(),
            start_time: now,
            end_time: now,
        },
    ]
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let n = std::env::args()
        .skip_while(|a| a != "--runs")
        .nth(1)
        .and_then(|a| a.parse::<usize>().ok())
        .unwrap_or(3);

    let llm = LlmClient::new(
        BASE.into(),
        API_KEY.into(),
        MODEL.into(),
        0.0,
        700,
        Some("low".into()),
    )?;
    let ctx = ContextBuilder::new(SYSTEM.into(), "Senior Rust developer".into(), 8000);

    let mut answered = 0usize;
    for run in 0..n {
        let messages = ctx.build(
            "",
            &turns(),
            Some(&turns()[0]),
            false,
            Some("помоги сформулировать"),
            None,
        );
        let (mut rx, handle) = llm.stream_answer(messages);
        let result = timeout(Duration::from_secs(90), async {
            let mut full = String::new();
            loop {
                match rx.recv().await {
                    Some(AnswerEvent::Token(t)) => full.push_str(&t),
                    Some(AnswerEvent::Done(_)) => return Ok(full),
                    Some(AnswerEvent::Failed(e)) => return Err(format!("failed: {e}")),
                    None => return Err("channel closed".to_string()),
                }
            }
        })
        .await;

        match result {
            Ok(Ok(full)) => {
                println!("[manual] run {run}: ANSWER: {full}");
                answered += 1;
            }
            Ok(Err(e)) => println!("[manual] run {run}: {e}"),
            Err(e) => println!("[manual] run {run}: timeout {e}"),
        }
        handle.abort();
    }

    println!("\n=== [manual] answered {answered}/{n} ===");
    if answered == 0 {
        std::process::exit(1);
    }
    Ok(())
}
