use crate::common::{DOWN, ENTER, ESC, TestDataDir};
#[test]
fn empty_damaged_or_unknown_selected_catalog_cannot_start_spanish_translation() {
    for state in ["empty", "damaged", "unknown"] {
        let data = TestDataDir::new();
        data.seed_wordset("abc");
        data.seed_translation("кот", "cat");
        data.seed_spanish(
            "ru_es_a1",
            if state == "damaged" {
                &[("", "casa")]
            } else {
                &[]
            },
        );
        let selected = if state == "unknown" {
            "missing_set"
        } else {
            "ru_es_a1"
        };
        data.write_settings(serde_json::json!({"quick_start_mode":"translation","quick_start_translation_set":selected}));
        let run = crate::common::run_wpm_capture(&[ENTER, ESC, ESC], data.as_ref());
        assert!(
            run.screens[0].contains("Vocabulary set unavailable or damaged"),
            "{}",
            run.screens[0]
        );
        assert!(!run.screens[0].contains("Word 1"));
        assert!(!run.screens[0].contains("Accuracy:"));
        assert!(run.screens[1].contains("Quick Start"));
    }
}
#[test]
fn corrupted_legacy_translation_json_is_an_error_instead_of_an_empty_success() {
    let data = TestDataDir::new();
    data.seed_wordset("abc");
    data.seed_translation("кот", "cat");
    rusqlite::Connection::open(data.as_ref().join("wordset.db"))
        .unwrap()
        .execute(
            "UPDATE translation_sets SET pairs_json='invalid json' WHERE name='ru_en_a1'",
            [],
        )
        .unwrap();
    let run = crate::common::run_wpm_capture(&[DOWN, ENTER, ESC, ESC], data.as_ref());
    assert!(run.screens[1].contains("Vocabulary set unavailable or damaged"));
    assert!(!run.screens[1].contains("Accuracy:"));
}
