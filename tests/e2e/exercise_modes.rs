use crate::common::{
    DOWN, ENTER, ESC, LEFT, PERFECT_ACCURACY, RIGHT, TestDataDir, UP, run_wpm_capture,
    run_wpm_capture_with_options,
};
use serde_json::{Value, json};
use std::{fs, os::unix::fs::PermissionsExt};

fn assert_labels(screen: &str, labels: &[&str]) {
    for label in labels {
        assert!(screen.contains(label), "missing {label:?}\n{screen}");
    }
    assert!(!screen.contains("Dictation"), "{screen}");
    assert!(!screen.contains("Words:"), "{screen}");
}

fn legacy_data(value: Value) -> TestDataDir {
    let data = TestDataDir::new();
    data.seed_wordset("unused");
    let conn = rusqlite::Connection::open(data.as_ref().join("wordset.db")).unwrap();
    conn.execute("UPDATE word_sets SET name='personal',words='кот'", [])
        .unwrap();
    data.seed_spanish("ru_es_a2", &[("дом", "casa")]);
    data.write_settings(json!({
        "quick_start_mode":"dictation", "quick_start_wordset":"personal",
        "quick_start_time":45, "quick_start_translation_set":"ru_es_a2",
        "translation_set":"ru_es_a2", "translation_time":90,
        "typing_layout_conversion":true,
        "dictation_set":value, "dictation_length":value,
        "quick_start_dictation_set":value, "quick_start_dictation_length":value
    }));
    data
}

fn assert_preferences(saved: &Value) {
    assert_eq!(saved["quick_start_mode"], "typing");
    assert_eq!(saved["quick_start_wordset"], "personal");
    assert_eq!(saved["quick_start_time"], 45);
    assert_eq!(saved["quick_start_translation_set"], "ru_es_a2");
    assert_eq!(saved["translation_set"], "ru_es_a2");
    assert_eq!(saved["translation_time"], 90);
    assert_eq!(saved["typing_layout_conversion"], true);
    for field in [
        "dictation_set",
        "dictation_length",
        "quick_start_dictation_set",
        "quick_start_dictation_length",
    ] {
        assert!(saved.get(field).is_none(), "{saved}");
    }
}

#[test]
fn all_mode_and_settings_choices_contain_only_text_exercises() {
    let data = TestDataDir::new();
    let run = run_wpm_capture(
        &[
            DOWN, DOWN, ENTER, LEFT, RIGHT, RIGHT, LEFT, ESC, DOWN, ENTER, LEFT, RIGHT, RIGHT,
            RIGHT, LEFT, LEFT, DOWN, LEFT, RIGHT, RIGHT, LEFT, ESC, ESC,
        ],
        data.as_ref(),
    );
    for index in [2, 3, 6] {
        assert_labels(&run.screens[index], &["< Typing >", "Time:"]);
    }
    for index in [4, 5] {
        assert_labels(&run.screens[index], &["< Translation >", "Time:"]);
    }
    assert_labels(&run.screens[9], &["< Quick Start >", "< typing >"]);
    assert_labels(&run.screens[11], &["< Translation >", "Dataset:"]);
    for index in [12, 13] {
        assert_labels(&run.screens[index], &["< Typing >", "Layout conversion:"]);
    }
    assert_labels(&run.screens[15], &["< Quick Start >"]);
    for index in [17, 20] {
        assert_labels(&run.screens[index], &["< typing >"]);
    }
    for index in [18, 19] {
        assert_labels(&run.screens[index], &["< translation >"]);
    }
}

#[test]
fn legacy_dictation_normalizes_without_writing_and_cancel_keeps_all_preferences() {
    for value in [
        json!("ru_es_a1"),
        json!(20),
        Value::Null,
        json!([1, "old"]),
        json!({"obsolete":true}),
        json!(false),
    ] {
        let data = legacy_data(value);
        let path = data.as_ref().join("settings.json");
        let before = fs::read(&path).unwrap();
        let run = run_wpm_capture(
            &[
                ENTER, b"rjn", ESC, DOWN, DOWN, DOWN, ENTER, RIGHT, RIGHT, DOWN, LEFT, ESC, ENTER,
                RIGHT, RIGHT, ESC, UP, UP, UP, ENTER, b"rjn", ESC, ESC,
            ],
            data.as_ref(),
        );
        assert_labels(&run.screens[0], &["кот", "0.0s"]);
        assert_labels(&run.screens[1], &[PERFECT_ACCURACY]);
        assert_labels(
            &run.screens[6],
            &["< typing >", "< personal >", "< 45 sec >"],
        );
        assert_labels(&run.screens[7], &["< ru_es_a2 >", "< 1 min 30 sec >"]);
        assert_labels(&run.screens[8], &["< On >"]);
        assert_labels(&run.screens[14], &["< On >"]);
        assert_labels(&run.screens[20], &[PERFECT_ACCURACY]);
        assert_eq!(
            fs::read(path).unwrap(),
            before,
            "read/cancel rewrote legacy settings"
        );
    }
}

#[test]
fn legacy_save_and_restart_remove_obsolete_fields_and_preserve_preferences() {
    let data = legacy_data(json!({"arbitrary": [false, null, "value"]}));
    run_wpm_capture(&[DOWN, DOWN, DOWN, ENTER, ENTER, ESC], data.as_ref());
    let saved: Value =
        serde_json::from_slice(&fs::read(data.as_ref().join("settings.json")).unwrap()).unwrap();
    assert_preferences(&saved);
    let run = run_wpm_capture(
        &[DOWN, DOWN, DOWN, ENTER, DOWN, RIGHT, ESC, ESC],
        data.as_ref(),
    );
    assert_labels(
        &run.screens[3],
        &["< typing >", "< personal >", "< 45 sec >"],
    );
    assert_labels(
        &run.screens[5],
        &["< translation >", "< ru_es_a2 >", "< 45 sec >"],
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(data.as_ref().join("settings.json")).unwrap())
            .unwrap(),
        saved
    );
}

#[test]
fn new_settings_save_only_supported_defaults() {
    let data = TestDataDir::new();
    run_wpm_capture(&[DOWN, DOWN, DOWN, ENTER, ENTER, ESC], data.as_ref());
    let saved: Value =
        serde_json::from_slice(&fs::read(data.as_ref().join("settings.json")).unwrap()).unwrap();
    assert_eq!(
        saved,
        json!({"quick_start_mode":"typing","quick_start_time":15,
        "quick_start_wordset":"en_1000","quick_start_translation_set":"ru_en_a1",
        "translation_time":60,"translation_set":"ru_en_a1","typing_layout_conversion":false})
    );
}

#[test]
fn obsolete_tts_environment_never_starts_a_program() {
    let data = TestDataDir::new();
    data.seed_wordset("abc");
    data.seed_translation("кот", "cat");
    data.write_settings(json!({}));
    let binary = data.as_ref().join("obsolete program");
    let marker = data.as_ref().join("was-started");
    fs::write(
        &binary,
        "#!/bin/sh\ntouch \"$(dirname \"$0\")/was-started\"\n",
    )
    .unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    let run = run_wpm_capture_with_options(
        &[
            UP, DOWN, DOWN, DOWN, DOWN, ENTER, RIGHT, RIGHT, ESC, UP, UP, UP, ENTER, b"abc", ENTER,
            b"abc", ESC, DOWN, ENTER, b"rjn cat", ENTER, b"rjn cat", ESC, ESC,
        ],
        data.as_ref(),
        &[("WPM_TTS_BIN", &binary)],
        &[],
    );
    for index in [13, 15, 19, 21] {
        assert_labels(&run.screens[index], &[PERFECT_ACCURACY]);
    }
    assert!(!marker.exists(), "obsolete speech executable was invoked");
}
