#![cfg(unix)]

mod common;

use common::{
    CTRL_C, DOWN, ENTER, ESC, EXIT, MENU_LABELS, QUICK_START, SELECT_MODE, SETTINGS, TRANSLATION,
    TestDataDir, UP, run_wmp_capture, run_wmp_with_keys,
};

#[test]
fn menu_shows_all_actions_and_exits_on_escape() {
    let data_dir = TestDataDir::new();

    let stdout = run_wmp_with_keys(&[ESC], data_dir.as_ref());
    for label in MENU_LABELS {
        assert!(
            stdout.contains(label),
            "expected menu to contain {label}\nstdout:\n{stdout}"
        );
    }
    assert!(
        stdout.contains(&format!("> {QUICK_START}")),
        "expected Quick Start to be selected initially\nstdout:\n{stdout}"
    );
}

#[test]
fn menu_selection_moves_in_both_directions_and_wraps() {
    let data_dir = TestDataDir::new();

    let stdout = run_wmp_with_keys(
        &[
            UP, DOWN, DOWN, DOWN, DOWN, DOWN, DOWN, UP, UP, UP, UP, UP, ESC,
        ],
        data_dir.as_ref(),
    );
    let expected_selections = [
        QUICK_START,
        EXIT,
        QUICK_START,
        TRANSLATION,
        SELECT_MODE,
        SETTINGS,
        EXIT,
        QUICK_START,
        EXIT,
        SETTINGS,
        SELECT_MODE,
        TRANSLATION,
        QUICK_START,
    ];
    let mut remaining_output = stdout.as_str();
    for label in expected_selections {
        let marker = format!("> {label}");
        let position = remaining_output
            .find(&marker)
            .unwrap_or_else(|| panic!("expected selection {marker} in order\nstdout:\n{stdout}"));
        remaining_output = &remaining_output[position + marker.len()..];
    }
}

#[test]
fn menu_exit_action_closes_application() {
    let data_dir = TestDataDir::new();
    let run = run_wmp_capture(&[UP, ENTER], data_dir.as_ref());

    assert!(
        run.screens[0].contains("> Exit"),
        "expected Exit to be selected\nscreen:\n{}",
        run.screens[0]
    );
}

#[test]
fn control_c_closes_application() {
    let data_dir = TestDataDir::new();
    run_wmp_with_keys(&[CTRL_C], data_dir.as_ref());
}
