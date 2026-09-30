#![cfg(unix)]

mod common;

use common::{
    DOWN, ENTER, ESC, PERFECT_ACCURACY, TEST_SOURCE, TEST_TRANSLATION, TEST_WORD, TestDataDir,
    run_wmp_capture, run_wmp_capture_with_pause,
};
use std::time::Duration;

#[test]
fn completed_typing_shows_result_and_returns_to_menu() {
    let data_dir = TestDataDir::new();
    data_dir.seed_wordset(TEST_WORD);
    let run = run_wmp_capture(&[ENTER, b"x\x7fabc", DOWN, ENTER, ESC], data_dir.as_ref());
    let result = &run.screens[1];

    for label in ["WPM:", PERFECT_ACCURACY, "Time:", "> Restart", "Menu"] {
        assert!(
            result.contains(label),
            "expected {label:?}\nscreen:\n{result}"
        );
    }
    assert!(
        run.screens[3].contains("> Quick Start"),
        "expected return to menu\nscreen:\n{}",
        run.screens[3]
    );
}

#[test]
fn result_can_restart_test() {
    let data_dir = TestDataDir::new();
    data_dir.seed_wordset(TEST_WORD);
    let run = run_wmp_capture(&[ENTER, b"abc", ENTER, b"abc", ESC, ESC], data_dir.as_ref());

    assert!(
        run.screens[1].contains(PERFECT_ACCURACY),
        "expected first completed attempt\nscreen:\n{}",
        run.screens[1]
    );
    assert!(
        run.screens[2].contains(TEST_WORD),
        "expected restart to show target\nscreen:\n{}",
        run.screens[2]
    );
    assert!(
        run.screens[3].contains(PERFECT_ACCURACY),
        "expected second completed attempt\nscreen:\n{}",
        run.screens[3]
    );
}

#[test]
fn completed_translation_shows_accurate_result() {
    let data_dir = TestDataDir::new();
    data_dir.seed_wordset(TEST_WORD);
    data_dir.seed_translation(TEST_SOURCE, TEST_TRANSLATION);
    let run = run_wmp_capture(&[DOWN, ENTER, b"rjn cat", ESC, ESC], data_dir.as_ref());

    assert!(
        run.screens[2].contains(PERFECT_ACCURACY),
        "expected translated input to be accepted\nscreen:\n{}",
        run.screens[2]
    );
    assert!(
        run.screens[3].contains("> Translation"),
        "expected Escape to return to menu\nscreen:\n{}",
        run.screens[3]
    );
}

#[test]
fn incorrect_input_reduces_accuracy() {
    let data_dir = TestDataDir::new();
    data_dir.seed_wordset(TEST_WORD);
    let run = run_wmp_capture(&[ENTER, b"xbc", ESC, ESC], data_dir.as_ref());

    assert!(
        run.screens[1].contains("Accuracy: 67%"),
        "expected one incorrect character\nscreen:\n{}",
        run.screens[1]
    );
}

#[test]
fn time_limit_finishes_an_incomplete_attempt() {
    let data_dir = TestDataDir::new();
    data_dir.seed_wordset("abcdefghij");
    let run = run_wmp_capture_with_pause(
        &[ENTER, b"a", ESC, ESC],
        data_dir.as_ref(),
        Some((1, Duration::from_secs(16))),
    );

    assert!(
        run.screens[1].contains(PERFECT_ACCURACY),
        "expected a result after the time limit\nscreen:\n{}",
        run.screens[1]
    );
    assert!(
        run.screens[2].contains("> Quick Start"),
        "expected Escape to return to menu\nscreen:\n{}",
        run.screens[2]
    );
}

#[test]
fn translation_result_can_restart() {
    let data_dir = TestDataDir::new();
    data_dir.seed_wordset(TEST_WORD);
    data_dir.seed_translation(TEST_SOURCE, TEST_TRANSLATION);
    let run = run_wmp_capture(
        &[DOWN, ENTER, b"rjn cat", ENTER, b"rjn cat", ESC, ESC],
        data_dir.as_ref(),
    );

    assert!(
        run.screens[2].contains(PERFECT_ACCURACY),
        "expected first translation result\nscreen:\n{}",
        run.screens[2]
    );
    assert!(
        run.screens[3].contains("кот cat"),
        "expected translation target after restart\nscreen:\n{}",
        run.screens[3]
    );
    assert!(
        run.screens[4].contains(PERFECT_ACCURACY),
        "expected second translation result\nscreen:\n{}",
        run.screens[4]
    );
}
