#![cfg(unix)]

mod common;

use common::{DOWN, ENTER, ESC, PERFECT_ACCURACY, RIGHT, TestDataDir, UP, run_wpm_capture};
use std::fs;

fn seeded_data() -> TestDataDir {
    let data_dir = TestDataDir::new();
    data_dir.seed_wordset("abc");
    data_dir.seed_translation("ж", "x");
    data_dir
}

#[test]
fn translation_converts_from_every_start_path_with_typing_option_off() {
    let data_dir = seeded_data();

    let menu = run_wpm_capture(&[DOWN, ENTER, b"; x", ESC, ESC], data_dir.as_ref());
    assert!(menu.screens[2].contains(PERFECT_ACCURACY));

    let quick_start = run_wpm_capture(
        &[
            DOWN, DOWN, DOWN, ENTER, DOWN, RIGHT, ENTER, DOWN, DOWN, ENTER, b"; x", ESC, ESC,
        ],
        data_dir.as_ref(),
    );
    assert!(
        quick_start.screens[10].contains(PERFECT_ACCURACY),
        "Quick Start Translation must convert\nscreen:\n{}",
        quick_start.screens[10]
    );

    let select_mode = run_wpm_capture(
        &[DOWN, DOWN, ENTER, RIGHT, ENTER, b"; x", ESC, ESC],
        data_dir.as_ref(),
    );
    assert!(
        select_mode.screens[5].contains(PERFECT_ACCURACY),
        "Select Mode Translation must convert\nscreen:\n{}",
        select_mode.screens[5]
    );
}

#[test]
fn translation_restart_keeps_conversion_and_direct_input_works() {
    let data_dir = seeded_data();
    let run = run_wpm_capture(
        &[DOWN, ENTER, b"; x", ENTER, "ж x".as_bytes(), ESC, ESC],
        data_dir.as_ref(),
    );
    assert!(run.screens[2].contains(PERFECT_ACCURACY));
    assert!(run.screens[3].contains("ж x"));
    assert!(run.screens[4].contains(PERFECT_ACCURACY));
}

#[test]
fn typing_defaults_to_layout_sensitive_in_both_start_paths_and_restart() {
    let data_dir = seeded_data();
    let quick_start = run_wpm_capture(
        &[ENTER, "фbc".as_bytes(), ENTER, "фbc".as_bytes(), ESC, ESC],
        data_dir.as_ref(),
    );
    assert!(quick_start.screens[1].contains("Accuracy: 67%"));
    assert!(quick_start.screens[3].contains("Accuracy: 67%"));

    let select_mode = run_wpm_capture(
        &[DOWN, DOWN, ENTER, ENTER, "фbc".as_bytes(), ESC, ESC],
        data_dir.as_ref(),
    );
    assert!(select_mode.screens[4].contains("Accuracy: 67%"));

    let direct = run_wpm_capture(&[ENTER, b"abc", ESC, ESC], data_dir.as_ref());
    assert!(direct.screens[1].contains(PERFECT_ACCURACY));
}

#[test]
fn typing_conversion_setting_is_visible_saved_and_used_by_both_start_paths() {
    let data_dir = seeded_data();
    let enabled = run_wpm_capture(
        &[
            DOWN,
            DOWN,
            DOWN,
            ENTER,
            RIGHT,
            RIGHT,
            DOWN,
            RIGHT,
            ENTER,
            DOWN,
            DOWN,
            ENTER,
            "фис".as_bytes(),
            ESC,
            ESC,
        ],
        data_dir.as_ref(),
    );
    assert!(enabled.screens[5].contains("< Typing >"));
    assert!(enabled.screens[7].contains("< On >"));
    assert!(enabled.screens[12].contains(PERFECT_ACCURACY));

    let reopened = run_wpm_capture(
        &[
            DOWN,
            DOWN,
            DOWN,
            ENTER,
            RIGHT,
            RIGHT,
            ESC,
            UP,
            ENTER,
            ENTER,
            "фис".as_bytes(),
            ESC,
            ESC,
        ],
        data_dir.as_ref(),
    );
    assert!(reopened.screens[5].contains("< On >"));
    assert!(reopened.screens[10].contains(PERFECT_ACCURACY));
}

#[test]
fn old_settings_file_defaults_typing_conversion_to_off() {
    let data_dir = seeded_data();
    let old_settings = serde_json::json!({
        "quick_start_mode": "typing",
        "quick_start_time": 15,
        "quick_start_wordset": "en_1000",
        "quick_start_translation_set": "ru_en_a1",
        "translation_time": 60,
        "translation_set": "ru_en_a1"
    });
    fs::write(
        data_dir.as_ref().join("settings.json"),
        old_settings.to_string(),
    )
    .expect("failed to write old settings");

    let run = run_wpm_capture(
        &[
            ENTER,
            "фbc".as_bytes(),
            ESC,
            DOWN,
            DOWN,
            DOWN,
            ENTER,
            RIGHT,
            RIGHT,
            ESC,
            ESC,
        ],
        data_dir.as_ref(),
    );
    assert!(run.screens[1].contains("Accuracy: 67%"));
    assert!(run.screens[8].contains("< Off >"));
}
