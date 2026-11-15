#![cfg(unix)]

mod common;

use common::{DOWN, ENTER, ESC, PERFECT_ACCURACY, TestDataDir, run_wpm_capture};

fn data(pairs: &[(&str, &str)]) -> TestDataDir {
    let data = TestDataDir::new();
    data.seed_wordset("abc");
    data.seed_spanish("ru_es_a1", pairs);
    data.write_settings(serde_json::json!({
        "translation_set": "ru_es_a1", "quick_start_translation_set": "ru_es_a1",
        "quick_start_mode": "translation"
    }));
    data
}

#[test]
fn spanish_translation_accepts_eng_ru_and_canonical_input_from_all_start_paths() {
    let data = data(&[("год", "año")]);
    for (start, answer) in [
        (vec![DOWN, ENTER], "ujl ano"),
        (vec![ENTER], "год фтщ"),
        (vec![DOWN, DOWN, ENTER, common::RIGHT, ENTER], "год año"),
    ] {
        let mut keys = start;
        keys.extend([answer.as_bytes(), ESC, ESC]);
        let run = run_wpm_capture(&keys, data.as_ref());
        assert!(
            run.screens[keys.len() - 3].contains(PERFECT_ACCURACY),
            "{}",
            run.transcript
        );
    }
}

#[test]
fn spanish_translation_supports_all_seven_diacritics_and_restart() {
    for (canonical, simple, russian) in [
        ("niño", "nino", "тштщ"),
        ("árbol", "arbol", "фкищд"),
        ("café", "cafe", "сфау"),
        ("rubí", "rubi", "кгиш"),
        ("jabón", "jabon", "офищт"),
        ("menú", "menu", "ьутг"),
        ("pingüino", "pinguino", "зштпгштщ"),
    ] {
        let data = data(&[("год", canonical)]);
        let eng = format!("ujl {simple}");
        let ru = format!("год {russian}");
        let direct = format!("год {canonical}");
        let run = run_wpm_capture(
            &[
                DOWN,
                ENTER,
                eng.as_bytes(),
                ENTER,
                ru.as_bytes(),
                ENTER,
                direct.as_bytes(),
                ESC,
                ESC,
            ],
            data.as_ref(),
        );
        for index in [2, 4, 6] {
            assert!(
                run.screens[index].contains(PERFECT_ACCURACY),
                "{canonical}: {}",
                run.screens[index]
            );
        }
    }
}

#[test]
fn spanish_translation_keeps_wrong_letters_as_errors() {
    let data = data(&[("год", "año")]);
    let run = run_wpm_capture(&[DOWN, ENTER, b"ujl eno", ESC, ESC], data.as_ref());
    assert!(
        run.screens[2].contains("Accuracy: 86%"),
        "{}",
        run.screens[2]
    );
}

#[test]
fn spanish_translation_enter_and_backspace_cross_pair_boundary() {
    let data = data(&[("год", "año"), ("год", "año")]);
    let run = run_wpm_capture(
        &[
            DOWN,
            ENTER,
            ENTER,
            b"ujl ano",
            ENTER,
            b"u",
            b"\x7f\x7f",
            ENTER,
            b"ujl ano",
            ESC,
            ESC,
        ],
        data.as_ref(),
    );
    assert!(
        run.screens[8].contains(PERFECT_ACCURACY),
        "{}",
        run.screens[8]
    );
}

#[test]
fn spanish_translation_is_available_without_audio_dependency() {
    let data = data(&[("год", "año")]);
    let missing = data.as_ref().join("nonexistent-audio");
    let run = common::run_wpm_capture_with_options(
        &[DOWN, ENTER, b"ujl ano", ESC, ESC],
        data.as_ref(),
        &[("WPM_TTS_BIN", &missing)],
        &[],
    );
    assert!(
        run.screens[2].contains(PERFECT_ACCURACY),
        "{}",
        run.screens[2]
    );
}

#[test]
fn spanish_translation_backspace_removes_a_whole_decomposed_letter() {
    let data = data(&[("год", "camión")]);
    let run = run_wpm_capture(
        &[
            DOWN,
            ENTER,
            "год camio\u{301}".as_bytes(),
            b"\x7f",
            b"on",
            ESC,
            ESC,
        ],
        data.as_ref(),
    );
    assert!(
        run.screens[4].contains(PERFECT_ACCURACY),
        "{}",
        run.screens[4]
    );
}

#[test]
fn final_combining_mark_has_same_result_in_one_or_separate_terminal_chunks() {
    for (translation, mark, accuracy) in [
        (false, "\u{301}", "Accuracy: 100%"),
        (true, "\u{308}", "Accuracy: 89%"),
    ] {
        for separate in [false, true] {
            let data = TestDataDir::new();
            data.seed_wordset("café");
            data.seed_translation("кот", "cat");
            data.seed_spanish("ru_es_a1", &[("кофе", "café")]);
            data.write_settings(serde_json::json!({"quick_start_mode":if translation{"translation"}else{"typing"},"quick_start_translation_set":"ru_es_a1"}));
            let base = if translation { "кофе cafe" } else { "cafe" };
            let joined = format!("{base}{mark}");
            let keys: Vec<&[u8]> = if separate {
                vec![ENTER, base.as_bytes(), mark.as_bytes(), ESC, ESC]
            } else {
                vec![ENTER, joined.as_bytes(), ESC, ESC]
            };
            let pauses = if separate {
                vec![(1, std::time::Duration::from_millis(200))]
            } else {
                vec![]
            };
            let run = common::run_wpm_capture_with_options(&keys, data.as_ref(), &[], &pauses);
            let result = &run.screens[if separate { 2 } else { 1 }];
            assert!(result.contains(accuracy), "{result}");
        }
    }
}
