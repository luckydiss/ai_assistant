mod common;

use engine_store::ReplayLogger;
use std::path::PathBuf;

#[test]
fn replay_log_roundtrip() {
    let dir: PathBuf = common::temp_dir("replay");
    let mut logger = ReplayLogger::open(dir.clone()).unwrap();

    logger
        .log(&engine_store::ReplayEvent::Segment {
            id: "s1".into(),
            lane: "I".into(),
            duration_ms: 1200,
        })
        .unwrap();
    logger
        .log(&engine_store::ReplayEvent::Transcript {
            id: "s1".into(),
            lane: "I".into(),
            text: "привет".into(),
        })
        .unwrap();
    logger
        .log(&engine_store::ReplayEvent::Turn {
            speaker: "Interviewer".into(),
            text: "привет".into(),
        })
        .unwrap();
    logger
        .log(&engine_store::ReplayEvent::Trigger {
            kind: "Auto".into(),
            focus: "привет".into(),
        })
        .unwrap();
    logger
        .log(&engine_store::ReplayEvent::Answer {
            outcome: "answered".into(),
            text: "ответ".into(),
            ttft_ms: 700,
        })
        .unwrap();
    logger.save_segment_wav("s1", &[0.0f32, 0.5, -0.5]).unwrap();

    let events = ReplayLogger::read_events(&dir).unwrap();
    assert_eq!(events.len(), 5);
    assert_eq!(
        events[0],
        engine_store::ReplayEvent::Segment {
            id: "s1".into(),
            lane: "I".into(),
            duration_ms: 1200,
        }
    );
    assert_eq!(
        events[4],
        engine_store::ReplayEvent::Answer {
            outcome: "answered".into(),
            text: "ответ".into(),
            ttft_ms: 700,
        }
    );
    assert!(dir.join("audio").join("s1.wav").exists());
    let wav = hound::WavReader::open(dir.join("audio").join("s1.wav")).unwrap();
    assert_eq!(wav.spec().sample_rate, 16000);
}

#[test]
fn replay_simulates_triggers() {
    use engine_dialogue::Speaker;
    use engine_orchestrator::{trigger_decision, TriggerKind};

    let now = chrono::Utc::now();
    let short = engine_dialogue::Turn {
        speaker: Speaker::Interviewer,
        text: "короткая".into(),
        start_time: now,
        end_time: now,
    
    typed: false,
    };
    let long_no_q = engine_dialogue::Turn {
        speaker: Speaker::Interviewer,
        text: "очень длинная реплика без вопроса которая занимает много слов".into(),
        start_time: now,
        end_time: now,
    
    typed: false,
    };
    let long_q = engine_dialogue::Turn {
        speaker: Speaker::Interviewer,
        text: "это очень длинный вопрос который содержит знак вопроса внутри текста и требует развернутого объяснения по существу дела?".into(),
        start_time: now,
        end_time: now,
    
    typed: false,
    };

    assert!(trigger_decision(&short, 4, 600).is_none());
    assert_eq!(
        trigger_decision(&long_no_q, 4, 600),
        Some((TriggerKind::Auto, 600))
    );
    assert_eq!(
        trigger_decision(&long_q, 4, 600),
        Some((TriggerKind::Speculative, 200))
    );
}

#[test]
fn replay_debounce_cancel() {
    use engine_dialogue::Speaker;
    use engine_orchestrator::trigger_decision;

    let now = chrono::Utc::now();
    let turns = vec![
        engine_dialogue::Turn {
            speaker: Speaker::Interviewer,
            text: "первый длинный вопрос без знака вопроса который занимает много слов".into(),
            start_time: now,
            end_time: now,
        
        typed: false,
        },
        engine_dialogue::Turn {
            speaker: Speaker::Interviewer,
            text: "второй длинный вопрос без знака вопроса который занимает много слов".into(),
            start_time: now,
            end_time: now,
        
        typed: false,
        },
    ];

    let mut fire_at: Option<u64> = None;
    let mut fired = 0;
    let mut seq: u64 = 0;
    for t in &turns {
        seq += 1;
        if let Some((_kind, delay)) = trigger_decision(t, 4, 600) {
            let at = seq * 1000 + delay;
            if let Some(prev) = fire_at {
                if at < prev {
                    continue;
                }
            }
            fire_at = Some(at);
            fired += 1;
        }
    }
    assert_eq!(fired, 2);
}
