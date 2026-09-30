#![cfg(unix)]

mod common;

use common::{
    DOWN, ENTER, ESC, TEST_SOURCE, TEST_TRANSLATION, TEST_TRANSLATION_TEXT, TEST_WORD, TestDataDir,
    run_wmp_capture,
};

#[test]
fn quick_start_shows_target_and_escape_returns_to_menu() {
    let data_dir = TestDataDir::new();
    data_dir.seed_wordset(TEST_WORD);
    let run = run_wmp_capture(&[ENTER, ESC, ESC], data_dir.as_ref());
    let typing_screen = &run.screens[0];

    assert!(
        typing_screen.contains(TEST_WORD),
        "expected target text\nscreen:\n{typing_screen}"
    );
    assert!(
        typing_screen.contains("0 wpm"),
        "expected speed counter\nscreen:\n{typing_screen}"
    );
    assert!(
        run.screens[1].contains("> Quick Start"),
        "expected return to menu\nscreen:\n{}",
        run.screens[1]
    );
}

#[test]
fn translation_starts_from_menu() {
    let data_dir = TestDataDir::new();
    data_dir.seed_wordset(TEST_WORD);
    data_dir.seed_translation(TEST_SOURCE, TEST_TRANSLATION);
    let run = run_wmp_capture(&[DOWN, ENTER, ESC, ESC], data_dir.as_ref());
    let typing_screen = &run.screens[1];

    assert!(
        typing_screen.contains(TEST_TRANSLATION_TEXT),
        "expected translation target\nscreen:\n{typing_screen}"
    );
    assert!(
        typing_screen.contains("wpm"),
        "expected typing screen\nscreen:\n{typing_screen}"
    );
}
