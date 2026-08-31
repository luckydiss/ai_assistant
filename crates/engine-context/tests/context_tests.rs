use chrono::{Duration, Utc};
use engine_context::{
    estimate_tokens, ChatMessage, ContextBuilder, ContextInput, MessageContent, PromptContext,
    Role,
};
use engine_dialogue::{Speaker, Turn};

fn turn(speaker: Speaker, text: &str, start_ms: i64) -> Turn {
    let start = Utc::now() + Duration::milliseconds(start_ms);
    Turn {
        speaker,
        text: text.to_string(),
        start_time: start,
        end_time: start + Duration::milliseconds(500),
        typed: false,
    }
}

fn text_of(msg: &ChatMessage) -> String {
    match &msg.content {
        MessageContent::Text(t) => t.clone(),
        MessageContent::Parts(_) => panic!("unexpected parts"),
    }
}

#[test]
fn context_input_fields() {
    let recent = [turn(Speaker::Interviewer, "r", 0)];
    let key = [turn(Speaker::Candidate, "k", 1)];
    let inp = ContextInput {
        summary: "S",
        key_turns: &key,
        recent: &recent,
        focus: None,
        focus_live: false,
        note: None,
        image_b64: None,
        manual: false,
    };
    assert_eq!(inp.summary, "S");
    assert_eq!(inp.key_turns.len(), 1);
    assert_eq!(inp.recent.len(), 1);
    assert!(inp.focus.is_none());
    assert!(inp.note.is_none());
    assert!(inp.image_b64.is_none());
    assert!(!inp.manual);
}

#[test]
fn builds_all_layers() {
    let builder = ContextBuilder::new("system".into(), "p".into(), 8000);
    let key = [turn(Speaker::Interviewer, "ключевой вопрос k1", 0)];
    let recent = [
        turn(Speaker::Interviewer, "реплика r1", 1000),
        turn(Speaker::Candidate, "реплика r2", 2000),
    ];
    let focus = turn(Speaker::Interviewer, "финальный вопрос", 3000);
    let inp = ContextInput {
        summary: "Резюме S",
        key_turns: &key,
        recent: &recent,
        focus: Some(&focus),
        focus_live: false,
        note: Some("заметка N"),
        image_b64: None,
        manual: false,
    };

    let messages = builder.build(&inp);
    assert_eq!(messages.len(), 2);
    let content = text_of(&messages[1]);
    let pos = |needle: &str| content.find(needle).expect(needle);
    assert!(pos("Резюме всей сессии: Резюме S") < pos("Ключевые моменты"));
    assert!(pos("Ключевые моменты") < pos("k1"));
    assert!(pos("k1") < pos("r1"));
    assert!(pos("r1") < pos("r2"));
    assert!(pos("r2") < pos("финальный вопрос"));
    assert!(pos("финальный вопрос") < pos("заметка N"));
    assert!(content.ends_with("Ответь по запросу кандидата."));
}

#[test]
fn skips_empty_layers() {
    let builder = ContextBuilder::new("system".into(), "p".into(), 8000);
    let inp = ContextInput::new(&[]);
    let messages = builder.build(&inp);
    let content = text_of(&messages[1]);
    assert!(!content.contains("Резюме всей сессии"));
    assert!(!content.contains("Ключевые моменты"));
    assert!(content.contains("Недавние реплики"));
}

#[test]
fn budget_safety() {
    let builder = ContextBuilder::new("system".into(), "p".into(), 8000);
    let recent: Vec<Turn> = (0..200)
        .map(|i| {
            turn(
                Speaker::Interviewer,
                &format!("длинная реплика номер {i} со многими словами тут"),
                i * 100,
            )
        })
        .collect();
    let inp = ContextInput {
        summary: "",
        key_turns: &[],
        recent: &recent,
        focus: None,
        focus_live: false,
        note: None,
        image_b64: None,
        manual: false,
    };

    let messages = builder.build(&inp);
    let total: usize = messages
        .iter()
        .map(|m| estimate_tokens(&text_of(m)))
        .sum();
    assert!(total <= 8000, "total={total}");
}

#[test]
fn manual_prompt_used_for_manual_requests() {
    let builder = ContextBuilder::new("auto system".into(), "p".into(), 8000)
        .with_manual_system("manual system".into());
    let inp_auto = ContextInput {
        manual: false,
        ..ContextInput::new(&[])
    };
    let inp_manual = ContextInput {
        manual: true,
        ..ContextInput::new(&[])
    };

    let sys_auto = text_of(&builder.build(&inp_auto)[0]);
    let sys_manual = text_of(&builder.build(&inp_manual)[0]);
    assert!(sys_auto.contains("auto system"));
    assert!(!sys_auto.contains("manual system"));
    assert!(sys_manual.contains("manual system"));
    assert!(!sys_manual.contains("auto system"));
}

#[test]
fn manual_prompt_fallback_to_system() {
    let builder = ContextBuilder::new("auto system".into(), "p".into(), 8000);
    let inp = ContextInput {
        manual: true,
        ..ContextInput::new(&[])
    };
    let sys = text_of(&builder.build(&inp)[0]);
    assert!(sys.contains("auto system"));
}

#[test]
fn builds_full_context() {
    let builder = ContextBuilder::new("system".into(), "Rust dev".into(), 8000);
    let turns = vec![
        turn(Speaker::Interviewer, "привет", 0),
        turn(Speaker::Candidate, "здравствуйте", 1000),
    ];
    let focus = turn(Speaker::Interviewer, "Как работает async?", 2000);
    let inp = ContextInput {
        summary: "S",
        key_turns: &[],
        recent: &turns,
        focus: Some(&focus),
        focus_live: false,
        note: None,
        image_b64: None,
        manual: false,
    };

    let messages = builder.build(&inp);

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

    let inp = ContextInput {
        note: Some("помоги"),
        ..ContextInput::new(&[])
    };

    let messages = builder.build(&inp);

    assert!(text_of(&messages[1]).contains("помоги"));
    assert!(!text_of(&messages[1]).contains("Последний вопрос I"));
}

#[test]
fn no_skip_instruction() {
    let builder = ContextBuilder::new("system".into(), "p".into(), 8000);
    let focus = turn(Speaker::Interviewer, "q", 0);
    let inp = ContextInput {
        focus: Some(&focus),
        ..ContextInput::new(&[])
    };

    let messages = builder.build(&inp);

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
    let inp = ContextInput {
        focus: Some(&focus),
        ..ContextInput::new(&turns)
    };

    let messages = builder.build(&inp);

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
    let inp = ContextInput {
        focus: Some(&focus),
        ..ContextInput::new(&turns)
    };

    let messages = builder.build(&inp);

    let content = text_of(&messages[1]);
    assert!(content.contains("привет"));
    assert!(content.contains("здравствуйте"));
}

#[test]
fn appends_note() {
    let builder = ContextBuilder::new("system".into(), "persona".into(), 8000);
    let focus = turn(Speaker::Interviewer, "вопрос", 0);
    let inp = ContextInput {
        focus: Some(&focus),
        note: Some("смотри на код в IDE"),
        ..ContextInput::new(&[])
    };

    let messages = builder.build(&inp);

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
    let messages = builder.build(&ContextInput::new(&[]));

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
    let messages = builder.build(&ContextInput::new(&[]));

    assert!(text_of(&messages[0]).starts_with("системный промпт"));
    assert!(!text_of(&messages[0]).contains("Резюме кандидата"));
    assert!(!text_of(&messages[0]).contains("Вакансия"));
}

#[test]
fn vision_payload_contains_image_and_text() {
    let builder = ContextBuilder::new("system".into(), "p".into(), 8000);

    let inp = ContextInput {
        note: Some("посмотри на экран"),
        image_b64: Some("QUJD"),
        ..ContextInput::new(&[])
    };
    let messages = builder.build(&inp);

    assert_eq!(messages.len(), 2);
    match &messages[1].content {
        MessageContent::Parts(parts) => {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0].kind, "image_url");
            assert!(parts[0]
                .image_url
                .as_ref()
                .unwrap()
                .url
                .starts_with("data:image/png;base64,"));
            assert_eq!(parts[1].kind, "text");
            assert!(parts[1].text.as_deref().unwrap().contains("посмотри на экран"));
        }
        MessageContent::Text(_) => panic!("expected parts for image"),
    }
}
