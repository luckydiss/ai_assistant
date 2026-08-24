use chrono::{DateTime, Duration, Utc};
use engine_dialogue::{Assembler, Speaker, Transcript};

fn transcript(
    base: DateTime<Utc>,
    speaker: Speaker,
    text: &str,
    start_ms: i64,
    duration_ms: u64,
) -> Transcript {
    Transcript {
        speaker,
        text: text.to_string(),
        start_time: base + Duration::milliseconds(start_ms),
        duration_ms,
    }
}

#[tokio::test]
async fn orders_by_timestamp() {
    let base = Utc::now();
    let t1 = transcript(base, Speaker::Interviewer, "Hello there", 1000, 500);
    let t2 = transcript(base, Speaker::Candidate, "Hi there", 500, 600);

    let mut a = Assembler::new();
    a.process_transcript(t1).await.unwrap();
    a.process_transcript(t2).await.unwrap();

    let turns = a.get_dialogue().turns;
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].text, "Hi there");
    assert_eq!(turns[1].text, "Hello there");
}

#[tokio::test]
async fn handles_same_timestamp() {
    let base = Utc::now();
    let t_i = transcript(base, Speaker::Interviewer, "Hello there", 500, 500);
    let t_c = transcript(base, Speaker::Candidate, "Hi there", 500, 500);

    let mut a = Assembler::new();
    a.process_transcript(t_i).await.unwrap();
    a.process_transcript(t_c).await.unwrap();

    let turns = a.get_dialogue().turns;
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].speaker, Speaker::Interviewer);
    assert_eq!(turns[1].speaker, Speaker::Candidate);
}

#[tokio::test]
async fn merges_short_pause() {
    let base = Utc::now();
    let t1 = transcript(base, Speaker::Interviewer, "Hello there", 0, 500);
    let t2 = transcript(base, Speaker::Interviewer, "how are you", 700, 400);

    let mut a = Assembler::new();
    a.process_transcript(t1).await.unwrap();
    a.process_transcript(t2).await.unwrap();

    let turns = a.get_dialogue().turns;
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0].text, "Hello there how are you");
    assert_eq!(turns[0].end_time, base + Duration::milliseconds(1100));
}

#[tokio::test]
async fn splits_long_pause() {
    let base = Utc::now();
    let t1 = transcript(base, Speaker::Interviewer, "Hello there", 0, 500);
    let t2 = transcript(base, Speaker::Interviewer, "next question", 1300, 400);

    let mut a = Assembler::new();
    a.process_transcript(t1).await.unwrap();
    a.process_transcript(t2).await.unwrap();

    let turns = a.get_dialogue().turns;
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].text, "Hello there");
    assert_eq!(turns[1].text, "next question");
}

#[tokio::test]
async fn filters_exact_duplicate() {
    let base = Utc::now();
    let t1 = transcript(base, Speaker::Interviewer, "Hello there", 0, 500);
    let t2 = transcript(base, Speaker::Interviewer, "Hello there", 1000, 500);

    let mut a = Assembler::new();
    a.process_transcript(t1).await.unwrap();
    a.process_transcript(t2).await.unwrap();

    let turns = a.get_dialogue().turns;
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0].text, "Hello there");
}

#[tokio::test]
async fn keeps_similar_text() {
    let base = Utc::now();
    let t1 = transcript(base, Speaker::Interviewer, "Hello world", 0, 500);
    let t2 = transcript(base, Speaker::Interviewer, "Hello world!", 1000, 500);

    let mut a = Assembler::new();
    a.process_transcript(t1).await.unwrap();
    a.process_transcript(t2).await.unwrap();

    let turns = a.get_dialogue().turns;
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].text, "Hello world");
    assert_eq!(turns[1].text, "Hello world!");
}

#[tokio::test]
async fn filters_short_utterance() {
    let base = Utc::now();
    let t = transcript(base, Speaker::Interviewer, "ок", 0, 500);

    let mut a = Assembler::new();
    let turn = a.process_transcript(t).await.unwrap();

    assert!(turn.is_none());
}

#[tokio::test]
async fn filters_filler_word() {
    let base = Utc::now();
    let mut a = Assembler::new();

    for word in ["ага", "хорошо"] {
        let t = transcript(base, Speaker::Interviewer, word, 0, 500);
        assert!(a.process_transcript(t).await.unwrap().is_none());
    }

    assert!(a.get_dialogue().turns.is_empty());
}

#[tokio::test]
async fn keeps_valid_short_reply() {
    let base = Utc::now();
    let t = transcript(base, Speaker::Candidate, "Да", 0, 500);

    let mut a = Assembler::new();
    a.process_transcript(t).await.unwrap();

    let turns = a.get_dialogue().turns;
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0].text, "Да");
}

#[tokio::test]
async fn custom_merge_threshold() {
    let base = Utc::now();
    let t1 = transcript(base, Speaker::Interviewer, "Hello there", 0, 500);
    let t2 = transcript(base, Speaker::Interviewer, "next question", 1200, 400);

    let mut a = Assembler::with_params(1500, 2, 16);
    a.process_transcript(t1).await.unwrap();
    a.process_transcript(t2).await.unwrap();

    let turns = a.get_dialogue().turns;
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0].text, "Hello there next question");
}

#[tokio::test]
async fn generates_summary() {
    let base = Utc::now();
    let mut a = Assembler::new();

    for i in 0..20 {
        let speaker = if i % 2 == 0 {
            Speaker::Interviewer
        } else {
            Speaker::Candidate
        };
        let t = transcript(base, speaker, &format!("utterance {i}"), i * 600, 400);
        a.process_transcript(t).await.unwrap();
    }

    let dialogue = a.get_dialogue();
    assert!(!dialogue.summary.is_empty());
    assert!(dialogue.turns.len() < 20);
}

#[tokio::test]
async fn updates_summary() {
    let base = Utc::now();
    let mut a = Assembler::new();

    for i in 0..20 {
        let speaker = if i % 2 == 0 {
            Speaker::Interviewer
        } else {
            Speaker::Candidate
        };
        let t = transcript(base, speaker, &format!("utterance {i}"), i * 600, 400);
        a.process_transcript(t).await.unwrap();
    }

    let first_summary = a.get_dialogue().summary;
    assert!(!first_summary.is_empty());

    for i in 20..36 {
        let speaker = if i % 2 == 0 {
            Speaker::Interviewer
        } else {
            Speaker::Candidate
        };
        let t = transcript(base, speaker, &format!("utterance {i}"), i * 600, 400);
        a.process_transcript(t).await.unwrap();
    }

    let dialogue = a.get_dialogue();
    assert!(dialogue.summary.starts_with(&first_summary));
    assert!(dialogue.summary.len() > first_summary.len());
}

#[tokio::test]
async fn echo_dropped() {
    let base = Utc::now();
    let i = transcript(base, Speaker::Interviewer, "как работает event loop", 0, 500);
    let c = transcript(base, Speaker::Candidate, "как работает event loop", 300, 500);

    let mut a = Assembler::new();
    a.process_transcript(i).await.unwrap();
    a.process_transcript(c).await.unwrap();

    let turns = a.get_dialogue().turns;
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0].speaker, Speaker::Interviewer);
}

#[tokio::test]
async fn own_speech_kept() {
    let base = Utc::now();
    let i = transcript(base, Speaker::Interviewer, "как работает event loop", 0, 500);
    let c = transcript(base, Speaker::Candidate, "ну это микрозадачи", 300, 500);

    let mut a = Assembler::new();
    a.process_transcript(i).await.unwrap();
    a.process_transcript(c).await.unwrap();

    let turns = a.get_dialogue().turns;
    assert_eq!(turns.len(), 2);
    assert!(turns.iter().any(|t| t.speaker == Speaker::Candidate));
}

#[test]
fn word_jaccard_matches() {
    let a = "как работает event loop";
    let b = "как работает event loop!";
    assert!(engine_dialogue::word_jaccard(a, b) >= 0.7);
}

#[test]
fn word_jaccard_differs() {
    let a = "как работает event loop";
    let b = "ну это микрозадачи";
    assert!(engine_dialogue::word_jaccard(a, b) < 0.7);
}
