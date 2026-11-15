#![cfg(unix)]

mod common;

use common::{DOWN, ENTER, ESC, FakeAudio, LEFT, RIGHT, TestDataDir, UP, run_wpm_capture_waiting};

fn data() -> TestDataDir {
    let data = TestDataDir::new();
    data.seed_wordset("abc");
    data.seed_spanish("ru_es_a1", &[("год", "año")]);
    data.seed_spanish("ru_es_a2", &[("дом", "casa")]);
    data.write_settings(serde_json::json!({"translation_set": "ru_es_a1"}));
    data
}

#[test]
fn dictation_settings_persist_independently_from_quick_start_and_cancel_draft() {
    let data = data();
    let audio = FakeAudio::new(&data, "success");
    // Save direct Dictation a2 / 10 while Quick Start remains Typing.
    let save = common::run_wpm_capture(
        &[
            DOWN, DOWN, DOWN, ENTER, RIGHT, RIGHT, RIGHT, DOWN, RIGHT, DOWN, LEFT, ENTER, ESC,
        ],
        data.as_ref(),
    );
    assert!(save.screens[10].contains("< 10 >"), "{}", save.screens[10]);
    let reopen = common::run_wpm_capture(
        &[
            DOWN, DOWN, DOWN, ENTER, RIGHT, RIGHT, RIGHT, DOWN, RIGHT, ESC, ENTER, RIGHT, RIGHT,
            RIGHT, ESC, ESC,
        ],
        data.as_ref(),
    );
    assert!(reopen.screens[3].contains("< typing >"));
    assert!(
        reopen.screens[13].contains("< ru_es_a2 >"),
        "{}",
        reopen.screens[13]
    );
    assert!(reopen.screens[13].contains("< 10 >"));
    let direct = run_wpm_capture_waiting(
        &[UP, UP, ENTER, b"casa", ENTER, ENTER, ESC, ESC],
        data.as_ref(),
        &[("WPM_TTS_BIN", &audio.binary)],
        &[],
        &[(2, "Audio ready")],
    );
    assert!(
        direct.screens[4].contains("Answer: casa"),
        "{}",
        direct.screens[4]
    );
    assert!(direct.screens[5].contains("Checked: 1"));
}

#[test]
fn quick_start_dictation_can_be_saved_and_select_mode_can_start_it() {
    let data = data();
    let audio = FakeAudio::new(&data, "success");
    common::run_wpm_capture(
        &[
            DOWN, DOWN, DOWN, ENTER, DOWN, RIGHT, RIGHT, DOWN, RIGHT, DOWN, RIGHT, ENTER, ESC,
        ],
        data.as_ref(),
    );
    let reopened = common::run_wpm_capture(&[DOWN, DOWN, DOWN, ENTER, ESC, ESC], data.as_ref());
    for label in ["< dictation >", "< ru_es_a2 >", "Words:", "< 50 >"] {
        assert!(
            reopened.screens[3].contains(label),
            "{}",
            reopened.screens[3]
        );
    }
    let quick = run_wpm_capture_waiting(
        &[ENTER, b"casa", ENTER, ENTER, ESC, ESC],
        data.as_ref(),
        &[("WPM_TTS_BIN", &audio.binary)],
        &[],
        &[(0, "Audio ready")],
    );
    assert!(
        quick.screens[2].contains("Answer: casa"),
        "{}",
        quick.screens[2]
    );
    let select = run_wpm_capture_waiting(
        &[
            DOWN, DOWN, ENTER, RIGHT, RIGHT, ENTER, b"ano", ENTER, ESC, ESC,
        ],
        data.as_ref(),
        &[("WPM_TTS_BIN", &audio.binary)],
        &[],
        &[(5, "Audio ready")],
    );
    assert!(
        select.screens[7].contains("Answer: año"),
        "{}",
        select.screens[7]
    );
}

#[test]
fn spanish_translation_setting_persists_and_unsaved_dataset_is_discarded() {
    let data = data();
    let save = common::run_wpm_capture(
        &[DOWN, DOWN, DOWN, ENTER, RIGHT, DOWN, RIGHT, ENTER, ESC],
        data.as_ref(),
    );
    assert!(save.screens[6].contains("< ru_es_a2 >"));
    let reopen = common::run_wpm_capture(
        &[
            DOWN,
            DOWN,
            DOWN,
            ENTER,
            RIGHT,
            DOWN,
            LEFT,
            ESC,
            ENTER,
            RIGHT,
            ESC,
            UP,
            UP,
            ENTER,
            b"ljv casa",
            ESC,
            ESC,
        ],
        data.as_ref(),
    );
    assert!(
        reopen.screens[9].contains("< ru_es_a2 >"),
        "{}",
        reopen.screens[9]
    );
    assert!(
        reopen.screens[14].contains(common::PERFECT_ACCURACY),
        "{}",
        reopen.screens[14]
    );
}
