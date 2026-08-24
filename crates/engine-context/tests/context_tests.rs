use chrono::{Duration, Utc};
use engine_context::{
    estimate_tokens, ChatMessage, ContextBuilder, MessageContent, PromptContext, Role,
};
use engine_dialogue::{Speaker, Turn};

fn turn(speaker: Speaker, text: &str, start_ms: i64) -> Turn {
    let start = Utc::now() + Duration::milliseconds(start_ms);
    Turn {
        speaker,
        text: text.to_string(),
        start_time: start,
        end_time: start + Duration::milliseconds(500),
    }
}

fn text_of(msg: &ChatMessage) -> String {
    match &msg.content {
        MessageContent::Text(t) => t.clone(),
        MessageContent::Parts(_) => panic!("unexpected parts"),
    }
}

#[test]
fn builds_full_context() {
    let builder = ContextBuilder::new("system".into(), "Rust dev".into(), 8000);
    let summary = "S";
    let turns = vec![
        turn(Speaker::Interviewer, "привет", 0),
        turn(Speaker::Candidate, "здравствуйте", 1000),
    ];
    let focus = turn(Speaker::Interviewer, "Как работает async?", 2000);

    let messages = builder.build(summary, &turns, Some(&focus), false, None, None);

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, Role::System);
    assert!(text_of(&messages[0]).contains("Rust dev"));
    assert_eq!(messages[1].role, Role::User);
    let content = text_of(&messages[1]);
    assert!(content.contains("S"));
    assert!(content.contains("привет"));
    assert!(content.contains("здравствуйте"));
    assert!(content.contains("Как работает async?"));
}

#[test]
fn builds_without_focus() {
    let builder = ContextBuilder::new("system".into(), "p".into(), 8000);

    let messages = builder.build("", &[], None, false, Some("помоги"), None);

    assert!(text_of(&messages[1]).contains("помоги"));
    assert!(!text_of(&messages[1]).contains("Последний вопрос I"));
}

#[test]
fn no_skip_instruction() {
    let builder = ContextBuilder::new("system".into(), "p".into(), 8000);
    let focus = turn(Speaker::Interviewer, "q", 0);

    let messages = builder.build("", &[], Some(&focus), false, None, None);

    assert!(!text_of(&messages[0]).contains("<SKIP>"));
    assert!(!text_of(&messages[1]).contains("<SKIP>"));
}

#[test]
fn truncates_oldest_turns() {
    let builder = ContextBuilder::new("system".into(), "persona".into(), 100);
    let turns: Vec<Turn> = (0..6)
        .map(|i| {
            let speaker = if i % 2 == 0 {
                Speaker::Interviewer
            } else {
                Speaker::Candidate
            };
            let text = format!("very long utterance number {i} with many words here");
            turn(speaker, &text, i * 1000)
        })
        .collect();
    let focus = turn(Speaker::Interviewer, "последний вопрос", 7000);

    let messages = builder.build("", &turns, Some(&focus), false, None, None);

    let content = text_of(&messages[1]);
    assert!(content.contains("последний вопрос"));
    assert!(content.contains("utterance number 5"));
    assert!(!content.contains("utterance number 0"));
}

#[test]
fn keeps_short_dialogue() {
    let builder = ContextBuilder::new("system".into(), "persona".into(), 8000);
    let turns = vec![
        turn(Speaker::Interviewer, "привет", 0),
        turn(Speaker::Candidate, "здравствуйте", 1000),
    ];
    let focus = turn(Speaker::Interviewer, "вопрос", 2000);

    let messages = builder.build("", &turns, Some(&focus), false, None, None);

    let content = text_of(&messages[1]);
    assert!(content.contains("привет"));
    assert!(content.contains("здравствуйте"));
}

#[test]
fn appends_note() {
    let builder = ContextBuilder::new("system".into(), "persona".into(), 8000);
    let focus = turn(Speaker::Interviewer, "вопрос", 0);

    let messages = builder.build("", &[], Some(&focus), false, Some("смотри на код в IDE"), None);

    assert!(text_of(&messages[1]).contains("смотри на код в IDE"));
}

#[test]
fn estimate_tokens_nonzero() {
    assert!(estimate_tokens("") > 0);
    assert!(estimate_tokens("a very long string with many characters here") > 0);
}

#[test]
fn builder_uses_context() {
    let ws = PromptContext {
        base_system: String::new(),
        role: "Senior Rust dev".into(),
        extra_prompt: "Отвечай только по существу".into(),
        resume_text: "5 лет в токенах".into(),
        vacancy: "Rust Backend".into(),
    };
    let builder = ContextBuilder::with_workspace("системный промпт".into(), &ws, 8000);
    let messages = builder.build("", &[], None, false, None, None);

    let system = text_of(&messages[0]);
    assert!(system.contains("системный промпт"));
    assert!(system.contains("Отвечай только по существу"));
    assert!(system.contains("Senior Rust dev"));
    assert!(system.contains("Резюме кандидата: 5 лет в токенах"));
    assert!(system.contains("Вакансия: Rust Backend"));
}

#[test]
fn builder_empty_context() {
    let ws = PromptContext::default();
    let builder = ContextBuilder::with_workspace("системный промпт".into(), &ws, 8000);
    let messages = builder.build("", &[], None, false, None, None);

    assert!(text_of(&messages[0]).starts_with("системный промпт"));
    assert!(!text_of(&messages[0]).contains("Резюме кандидата"));
    assert!(!text_of(&messages[0]).contains("Вакансия"));
}

#[test]
fn vision_payload_contains_image_and_text() {
    let builder = ContextBuilder::new("system".into(), "p".into(), 8000);

    let messages = builder.build("", &[], None, false, Some("посмотри на экран"), Some("QUJD"));

    assert_eq!(messages.len(), 2);
    match &messages[1].content {
        MessageContent::Parts(parts) => {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0].kind, "image_url");
            assert!(parts[0].image_url.as_ref().unwrap().url.starts_with("data:image/png;base64,"));
            assert_eq!(parts[1].kind, "text");
            assert!(parts[1].text.as_deref().unwrap().contains("посмотри на экран"));
        }
        MessageContent::Text(_) => panic!("expected parts for image"),
    }
}