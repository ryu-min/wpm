#![cfg(unix)]

mod common;

use common::{
    DOWN, ENTER, ESC, LEFT, RIGHT, SELECT_MODE, TEST_SOURCE, TEST_TRANSLATION,
    TEST_TRANSLATION_TEXT, TEST_WORD, TestDataDir, UP, run_wpm_capture,
};

#[test]
fn mode_selection_shows_defaults_and_returns_to_menu() {
    let data_dir = TestDataDir::new();
    let run = run_wpm_capture(&[DOWN, DOWN, ENTER, ESC, ESC], data_dir.as_ref());
    let mode_screen = &run.screens[2];

    for label in [
        SELECT_MODE,
        "Mode:",
        "< Typing >",
        "Wordset:",
        "en_1000 >",
        "Time:",
        "< 15 sec >",
    ] {
        assert!(
            mode_screen.contains(label),
            "expected {label:?}\nscreen:\n{mode_screen}"
        );
    }
    assert!(
        run.screens[3].contains("> Select Mode"),
        "expected return to menu\nscreen:\n{}",
        run.screens[3]
    );
}

#[test]
fn mode_selection_starts_typing_with_selected_wordset() {
    let data_dir = TestDataDir::new();
    data_dir.seed_wordset(TEST_WORD);
    let run = run_wpm_capture(&[DOWN, DOWN, ENTER, ENTER, ESC, ESC], data_dir.as_ref());
    let typing_screen = &run.screens[3];

    assert!(
        typing_screen.contains(TEST_WORD),
        "expected selected typing text\nscreen:\n{typing_screen}"
    );
    assert!(
        typing_screen.contains("wpm"),
        "expected typing screen\nscreen:\n{typing_screen}"
    );
}

#[test]
fn mode_selection_switches_to_translation_and_starts_it() {
    let data_dir = TestDataDir::new();
    data_dir.seed_wordset(TEST_WORD);
    data_dir.seed_translation(TEST_SOURCE, TEST_TRANSLATION);
    let run = run_wpm_capture(
        &[DOWN, DOWN, ENTER, RIGHT, ENTER, ESC, ESC],
        data_dir.as_ref(),
    );

    assert!(
        run.screens[3].contains("Translation set:"),
        "expected translation mode\nscreen:\n{}",
        run.screens[3]
    );
    assert!(
        run.screens[4].contains(TEST_TRANSLATION_TEXT),
        "expected translation text\nscreen:\n{}",
        run.screens[4]
    );
    assert!(
        run.screens[4].contains("wpm"),
        "expected typing screen\nscreen:\n{}",
        run.screens[4]
    );
}

#[test]
fn mode_selection_changes_mode_dataset_and_time() {
    let data_dir = TestDataDir::new();
    let run = run_wpm_capture(
        &[
            DOWN, DOWN, ENTER, UP, RIGHT, DOWN, RIGHT, DOWN, RIGHT, LEFT, ESC, ESC,
        ],
        data_dir.as_ref(),
    );

    assert!(
        run.screens[4].contains("< 30 sec >"),
        "expected time to change\nscreen:\n{}",
        run.screens[4]
    );
    assert!(
        run.screens[6].contains("< Translation >"),
        "expected mode to change\nscreen:\n{}",
        run.screens[6]
    );
    assert!(
        run.screens[8].contains("< ru_en_a2 >"),
        "expected translation set to change\nscreen:\n{}",
        run.screens[8]
    );
    assert!(
        run.screens[9].contains("< ru_en_a1 >"),
        "expected translation set to move left\nscreen:\n{}",
        run.screens[9]
    );
}

#[test]
fn mode_selection_changes_wordset() {
    let data_dir = TestDataDir::new();
    let run = run_wpm_capture(
        &[DOWN, DOWN, ENTER, DOWN, RIGHT, LEFT, ESC, ESC],
        data_dir.as_ref(),
    );

    assert!(
        run.screens[4].contains("< en_10000 >"),
        "expected next wordset\nscreen:\n{}",
        run.screens[4]
    );
    assert!(
        run.screens[5].contains("< en_1000 >"),
        "expected previous wordset\nscreen:\n{}",
        run.screens[5]
    );
}
