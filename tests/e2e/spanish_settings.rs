use crate::common::{DOWN, ENTER, ESC, LEFT, RIGHT, TestDataDir, UP};

fn data() -> TestDataDir {
    let data = TestDataDir::new();
    data.seed_wordset("abc");
    data.seed_spanish("ru_es_a1", &[("год", "año")]);
    data.seed_spanish("ru_es_a2", &[("дом", "casa")]);
    data.write_settings(serde_json::json!({"translation_set": "ru_es_a1"}));
    data
}

#[test]
fn spanish_translation_setting_persists_and_unsaved_dataset_is_discarded() {
    let data = data();
    let save = crate::common::run_wpm_capture(
        &[DOWN, DOWN, DOWN, ENTER, RIGHT, DOWN, RIGHT, ENTER, ESC],
        data.as_ref(),
    );
    assert!(save.screens[6].contains("< ru_es_a2 >"));
    let reopen = crate::common::run_wpm_capture(
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
        reopen.screens[14].contains(crate::common::PERFECT_ACCURACY),
        "{}",
        reopen.screens[14]
    );
}
