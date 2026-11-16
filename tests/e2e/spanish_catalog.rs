use crate::common::{DOWN, ENTER, ESC, TestDataDir};
#[test]
fn empty_or_damaged_selected_catalog_cannot_start_either_spanish_mode() {
    for damaged in [false, true] {
        for dictation in [false, true] {
            let data = TestDataDir::new();
            data.seed_wordset("abc");
            data.seed_translation("кот", "cat");
            data.seed_spanish("ru_es_a1", if damaged { &[("", "casa")] } else { &[] });
            data.write_settings(serde_json::json!({"quick_start_mode":if dictation{"dictation"}else{"translation"},"quick_start_translation_set":"ru_es_a1"}));
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
