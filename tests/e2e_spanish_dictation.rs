#![cfg(unix)]

mod common;

use common::{DOWN, ENTER, ESC, FakeAudio, TestDataDir, UP, run_wpm_capture_with_options};
use std::time::{Duration, Instant};

const F2: &[u8] = b"\x1bOQ";
const WAIT: Duration = Duration::from_millis(300);

fn data(words: &[(&str, &str)]) -> TestDataDir {
    let data = TestDataDir::new();
    data.seed_wordset("abc");
    data.seed_translation("кот", "cat");
    data.seed_spanish("ru_es_a1", words);
    data.write_settings(serde_json::json!({"quick_start_mode": "dictation"}));
    data
}

fn run(keys: &[&[u8]], data: &TestDataDir, audio: &FakeAudio, waits: &[usize]) -> common::TestRun {
    let mode = std::fs::read_to_string(&audio.control).unwrap();
    let expectations: Vec<_> = waits
        .iter()
        .enumerate()
        .map(|(i, index)| {
            (
                *index,
                match mode.as_str() {
                    "missing-voice" => "Spanish voice es unavailable",
                    "fail-first" if i == 0 => "Audio failed",
                    "delay" => "Playing",
                    _ => "Audio ready",
                },
            )
        })
        .collect();
    common::run_wpm_capture_waiting(
        keys,
        data.as_ref(),
        &[("WPM_TTS_BIN", &audio.binary)],
        &[],
        &expectations,
    )
}

fn contains(screen: &str, labels: &[&str]) {
    for label in labels {
        assert!(screen.contains(label), "expected {label:?}\n{screen}");
    }
}

#[test]
fn dictation_hides_target_until_submission_and_reveals_canonical_answer() {
    let data = data(&[("год", "año")]);
    let audio = FakeAudio::new(&data, "success");
    let run = run(
        &[UP, UP, ENTER, b"an", b"o", ENTER, ENTER, ESC, ESC],
        &data,
        &audio,
        &[2],
    );
    contains(&run.screens[3], &["Dictation", "an", "Checked: 0"]);
    for secret in ["año", "год", "Answer:", "Translation:"] {
        assert!(
            !run.screens[3].contains(secret),
            "leaked {secret}: {}",
            run.screens[3]
        );
    }
    contains(
        &run.screens[5],
        &[
            "Correct",
            "Answer: año",
            "Translation: год",
            "Your answer: ano",
            "Errors: 0",
        ],
    );
    contains(
        &run.screens[6],
        &[
            "Dictation result",
            "Checked: 1",
            "Correct: 1",
            "Errors: 0",
            "Correct words: 100%",
        ],
    );
    assert!(!run.screens[6].contains("WPM"));
    let starts: Vec<_> = audio
        .events()
        .into_iter()
        .filter(|e| e["event"] == "start")
        .collect();
    assert_eq!(starts.len(), 1);
    assert_eq!(starts[0]["word"], "año");
    assert_eq!(
        starts[0]["args"],
        serde_json::json!(["--stdin", "-b", "1", "-v", "es", "-s", "150"])
    );
}

#[test]
fn dictation_accepts_eng_ru_direct_and_colliding_spellings() {
    for (canonical, answer) in [
        ("año", "фтщ"),
        ("niño", "nino"),
        ("sí", "si"),
        ("si", "si"),
        ("año", "ano"),
        ("ano", "ano"),
        ("café", "café"),
    ] {
        let data = data(&[("перевод", canonical)]);
        let audio = FakeAudio::new(&data, "success");
        let run = run(
            &[ENTER, answer.as_bytes(), ENTER, ESC, ESC],
            &data,
            &audio,
            &[0],
        );
        contains(
            &run.screens[2],
            &["Correct", &format!("Answer: {canonical}"), "Errors: 0"],
        );
    }
}

#[test]
fn dictation_counts_words_once_correction_repeat_and_restart() {
    let data = data(&[("год", "año"), ("год", "año"), ("год", "año")]);
    let audio = FakeAudio::new(&data, "success");
    let run = run(
        &[
            ENTER, ENTER, b"ano", ENTER, F2, ENTER, b"xxx", ENTER, ENTER, b"anx", b"\x7f", b"o",
            ENTER, ENTER, ENTER, ESC, ESC,
        ],
        &data,
        &audio,
        &[0, 5, 8, 14],
    );
    contains(&run.screens[1], &["Checked: 0", "Errors: 0"]);
    contains(
        &run.screens[7],
        &[
            "Incorrect",
            "Answer: año",
            "Your answer: xxx",
            "Checked: 2",
            "Errors: 1",
        ],
    );
    contains(
        &run.screens[13],
        &[
            "Dictation result",
            "Checked: 3",
            "Correct: 2",
            "Errors: 1",
            "Correct words: 67%",
        ],
    );
    contains(
        &run.screens[14],
        &["Word 1 / 3", "Checked: 0", "Correct: 0", "Errors: 0"],
    );
    contains(&run.screens[15], &["Quick Start", "Translation"]);
}

#[test]
fn dictation_audio_failure_preserves_input_and_retry_recovers_without_word_error() {
    let data = data(&[("год", "año")]);
    let audio = FakeAudio::new(&data, "fail-first");
    let run = run(
        &[ENTER, b"ano", ENTER, F2, ENTER, ENTER, ESC, ESC],
        &data,
        &audio,
        &[0, 3],
    );
    contains(&run.screens[2], &["ano", "Checked: 0", "Errors: 0"]);
    assert!(!run.screens[2].contains("año"), "{}", run.screens[2]);
    assert!(!run.transcript.contains("secret diagnostic"));
    contains(&run.screens[4], &["Correct", "Answer: año", "Errors: 0"]);
    contains(
        &run.screens[5],
        &["Dictation result", "Correct words: 100%"],
    );
    assert_eq!(
        audio
            .events()
            .iter()
            .filter(|e| e["event"] == "start")
            .count(),
        2
    );
}

#[test]
fn dictation_delayed_audio_does_not_block_typing_repeat_or_escape() {
    let data = data(&[("год", "año")]);
    let audio = FakeAudio::new(&data, "delay");
    let started = Instant::now();
    let run = run(
        &[ENTER, b"ano", ENTER, F2, F2, ESC, ESC],
        &data,
        &audio,
        &[0],
    );
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "audio blocked input/exit"
    );
    contains(&run.screens[2], &["ano", "Checked: 0", "Errors: 0"]);
    assert!(!run.screens[2].contains("Answer:"));
    contains(&run.screens[5], &["Quick Start"]);
    let events = audio.events();
    assert_eq!(events.iter().filter(|e| e["event"] == "start").count(), 3);
    assert!(
        !events.iter().any(|e| e["event"] == "done"),
        "cancelled playback completed"
    );
}

#[test]
fn dictation_without_spanish_voice_does_not_start_and_other_modes_work() {
    let data = data(&[("год", "año")]);
    let audio = FakeAudio::new(&data, "missing-voice");
    let run = run(&[ENTER, ESC, DOWN, ENTER, ESC, ESC], &data, &audio, &[0]);
    contains(&run.screens[0], &["Spanish voice es unavailable"]);
    assert!(!run.screens[0].contains("Word 1"));
    assert!(!audio.events().iter().any(|e| e["event"] == "start"));
    contains(&run.screens[3], &["кот", "cat"]);
}

#[test]
fn dictation_ctrl_c_cancels_delayed_sound_and_exits() {
    let data = data(&[("год", "año")]);
    let audio = FakeAudio::new(&data, "delay");
    let started = Instant::now();
    let _ = run(&[ENTER, common::CTRL_C], &data, &audio, &[0]);
    assert!(started.elapsed() < Duration::from_secs(3));
    assert!(!audio.events().iter().any(|e| e["event"] == "done"));
}

#[test]
fn dictation_missing_program_reports_setup_and_can_return_to_menu() {
    let data = data(&[("год", "año")]);
    let missing = data.as_ref().join("missing speech program");
    let run = run_wpm_capture_with_options(
        &[ENTER, ESC, ESC],
        data.as_ref(),
        &[("WPM_TTS_BIN", &missing)],
        &[(0, WAIT)],
    );
    contains(&run.screens[0], &["install eSpeak NG", "WPM_TTS_BIN"]);
    assert!(!run.screens[0].contains("Word 1"));
    contains(&run.screens[1], &["Quick Start"]);
}

#[test]
fn dictation_trims_only_outer_spaces_and_preserves_case_checking() {
    for (answer, accepted) in [("  ano  ", true), ("a no", false), ("Ano", false)] {
        let data = data(&[("год", "año")]);
        let audio = FakeAudio::new(&data, "success");
        let run = run(
            &[ENTER, answer.as_bytes(), ENTER, ENTER, ESC, ESC],
            &data,
            &audio,
            &[0],
        );
        contains(
            &run.screens[3],
            if accepted {
                &["Correct: 1", "Errors: 0"]
            } else {
                &["Correct: 0", "Errors: 1"]
            },
        );
    }
}

#[test]
fn checked_answer_shows_failed_repeat_and_successful_recovery() {
    let data = data(&[("год", "año")]);
    let audio = FakeAudio::new(&data, "fail-second");
    let run = common::run_wpm_capture_waiting(
        &[ENTER, b"ano", ENTER, F2, F2, ENTER, ESC, ESC],
        data.as_ref(),
        &[("WPM_TTS_BIN", &audio.binary)],
        &[],
        &[(0, "Audio ready"), (3, "Audio failed"), (4, "Audio ready")],
    );
    for index in [2, 3, 4] {
        contains(
            &run.screens[index],
            &["Correct", "Answer: año", "Checked: 1", "Errors: 0"],
        );
    }
    contains(&run.screens[3], &["Audio failed", "F2: retry"]);
    contains(&run.screens[4], &["Audio ready"]);
    contains(
        &run.screens[5],
        &["Dictation result", "Correct: 1", "Errors: 0"],
    );
}

#[test]
fn ctrl_c_stops_owned_audio_before_returning_to_a_live_shell() {
    for mode in ["delay-one", "delay-readiness"] {
        let data = data(&[("год", "año")]);
        let audio = FakeAudio::new(&data, mode);
        let pid = data.as_ref().join("audio.pid");
        let run = common::run_wpm_capture_waiting(
            &[ENTER, common::CTRL_C],
            data.as_ref(),
            &[
                ("WPM_TTS_BIN", &audio.binary),
                ("WPM_TEST_HOLD_SHELL", &pid),
            ],
            &[(0, Duration::from_millis(250))],
            &[(
                0,
                if mode == "delay-readiness" {
                    "Checking Spanish audio"
                } else {
                    "Playing"
                },
            )],
        );
        assert!(run.transcript.contains("WPM_EXITED"));
        assert!(run.transcript.contains("SHELL_FINISHED"));
        assert!(
            !run.transcript.contains("TTS_STILL_ALIVE"),
            "{}",
            run.transcript
        );
        assert!(
            !audio
                .events()
                .iter()
                .any(|event| event["event"] == "done" || event["event"] == "readiness-done"),
            "{:?}",
            audio.events()
        );
    }
}

#[test]
fn session_lengths_sample_distinct_real_entries_and_finish_at_selected_length() {
    let words:Vec<_>="casa perro gato mesa silla puerta ventana libro papel lápiz agua leche pan queso arroz carne pescado huevo fruta manzana pera naranja limón uva plátano tomate patata cebolla ajo sal azúcar aceite café té sopa plato vaso taza cuchara tenedor cuchillo cama ropa zapato camisa abrigo sombrero calle ciudad pueblo coche tren avión barco bicicleta".split_whitespace().map(|word|("перевод",word)).collect();
    for length in [10usize, 20, 50] {
        let data = data(&words);
        data.write_settings(serde_json::json!({"quick_start_mode":"dictation","quick_start_dictation_length":length}));
        let audio = FakeAudio::new(&data, "success");
        let mut keys: Vec<&[u8]> = vec![ENTER];
        let mut expectations = vec![(0, "Audio ready")];
        for index in 0..length {
            keys.extend([b"wrong".as_slice(), ENTER, ENTER]);
            if index + 1 < length {
                expectations.push((keys.len() - 1, "Audio ready"));
            }
        }
        keys.extend([ESC, ESC]);
        let run = common::run_wpm_capture_waiting(
            &keys,
            data.as_ref(),
            &[("WPM_TTS_BIN", &audio.binary)],
            &[],
            &expectations,
        );
        contains(
            &run.screens[length * 3],
            &[
                "Dictation result",
                &format!("Checked: {length}"),
                &format!("Errors: {length}"),
            ],
        );
        let spoken: Vec<_> = audio
            .events()
            .iter()
            .filter(|e| e["event"] == "start")
            .map(|e| e["word"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(spoken.len(), length);
        assert_eq!(
            spoken
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            length
        );
        let revealed: std::collections::BTreeSet<_> = (0..length)
            .map(|index| {
                run.screens[index * 3 + 2]
                    .lines()
                    .find(|line| line.contains("Answer: "))
                    .unwrap()
                    .trim()
                    .to_string()
            })
            .collect();
        assert_eq!(revealed.len(), length);
    }
}
