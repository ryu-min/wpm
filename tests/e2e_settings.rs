#![cfg(unix)]

mod common;

use common::{
    DOWN, ENTER, ESC, RIGHT, SETTINGS, TEST_SOURCE, TEST_TRANSLATION, TEST_TRANSLATION_TEXT,
    TEST_WORD, TestDataDir, run_wpm_capture, run_wpm_with_keys,
};

#[test]
fn settings_show_both_sections() {
    let data_dir = TestDataDir::new();
    let run = run_wpm_capture(
        &[DOWN, DOWN, DOWN, ENTER, RIGHT, ESC, ESC],
        data_dir.as_ref(),
    );
    let quick_start = &run.screens[3];

    for label in [
        SETTINGS,
        "Configure:",
        "< Quick Start >",
        "Mode:",
        "Wordset:",
        "< en_1000 >",
        "< 15 sec >",
    ] {
        assert!(
            quick_start.contains(label),
            "expected {label:?}\nscreen:\n{quick_start}"
        );
    }
    let translation = &run.screens[4];
    for label in ["< Translation >", "Dataset:", "< ru_en_a1 >", "< 1 min >"] {
        assert!(
            translation.contains(label),
            "expected {label:?}\nscreen:\n{translation}"
        );
    }
}

#[test]
fn settings_save_and_load_on_next_launch() {
    let data_dir = TestDataDir::new();
    let change_settings = [
        DOWN, DOWN, DOWN, ENTER, DOWN, RIGHT, DOWN, RIGHT, DOWN, RIGHT, ENTER, ESC,
    ];
    run_wpm_with_keys(&change_settings, data_dir.as_ref());
    assert!(data_dir.as_ref().join("wordset.db").is_file());
    assert!(data_dir.as_ref().join("settings.json").is_file());

    let reopen_settings = [DOWN, DOWN, DOWN, ENTER, ESC, ESC];
    let run = run_wpm_capture(&reopen_settings, data_dir.as_ref());
    let screen = &run.screens[3];

    for value in [
        "< translation >",
        "Translation set:",
        "< ru_en_a2 >",
        "< 30 sec >",
    ] {
        assert!(
            screen.contains(value),
            "expected saved value {value:?}\nscreen:\n{screen}"
        );
    }
}

#[test]
fn escape_discards_unsaved_settings() {
    let data_dir = TestDataDir::new();
    let keys = [DOWN, DOWN, DOWN, ENTER, DOWN, RIGHT, ESC, ENTER, ESC, ESC];

    let run = run_wpm_capture(&keys, data_dir.as_ref());

    assert!(
        run.screens[7].contains("< typing >"),
        "expected original mode after reopening\nscreen:\n{}",
        run.screens[7]
    );
}

#[test]
fn translation_settings_save_independently() {
    let data_dir = TestDataDir::new();
    let keys = [
        DOWN, DOWN, DOWN, ENTER, RIGHT, DOWN, RIGHT, DOWN, RIGHT, ENTER, ESC,
    ];
    run_wpm_with_keys(&keys, data_dir.as_ref());

    let reopen = run_wpm_capture(
        &[DOWN, DOWN, DOWN, ENTER, RIGHT, ESC, ESC],
        data_dir.as_ref(),
    );
    let screen = &reopen.screens[4];
    assert!(
        screen.contains("< ru_en_a2 >"),
        "expected saved translation set\nscreen:\n{screen}"
    );
    assert!(
        screen.contains("< 1 min 30 sec >"),
        "expected saved translation time\nscreen:\n{screen}"
    );
}

#[test]
fn saved_quick_start_translation_is_used() {
    let data_dir = TestDataDir::new();
    data_dir.seed_wordset(TEST_WORD);
    data_dir.seed_translation(TEST_SOURCE, TEST_TRANSLATION);
    let keys = [
        DOWN, DOWN, DOWN, ENTER, DOWN, RIGHT, ENTER, DOWN, DOWN, ENTER, ESC, ESC,
    ];

    let run = run_wpm_capture(&keys, data_dir.as_ref());
    let screen = &run.screens[9];
    assert!(
        screen.contains(TEST_TRANSLATION_TEXT),
        "expected saved Quick Start mode to use translations\nscreen:\n{screen}"
    );
}

#[test]
fn quick_start_wordset_can_be_changed_and_saved() {
    let data_dir = TestDataDir::new();
    run_wpm_with_keys(
        &[DOWN, DOWN, DOWN, ENTER, DOWN, DOWN, RIGHT, ENTER, ESC],
        data_dir.as_ref(),
    );

    let run = run_wpm_capture(&[DOWN, DOWN, DOWN, ENTER, ESC, ESC], data_dir.as_ref());
    assert!(
        run.screens[3].contains("< en_10000 >"),
        "expected saved wordset\nscreen:\n{}",
        run.screens[3]
    );
}
