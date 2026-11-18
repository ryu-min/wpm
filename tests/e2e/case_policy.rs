use crate::common::*;
use serde_json::json;
use std::{fs, time::Duration};

fn fixture(word: &str, ru: &str, en: &str) -> TestDataDir {
    let data = TestDataDir::new();
    data.seed_wordset(word);
    data.seed_translation(ru, en);
    data
}

// Exercise the real settings UI, leaving all existing selections intact.
fn configure(data: &TestDataDir, typing: bool, translation: bool, layout: bool) {
    run_wpm_capture(
        &[
            DOWN,
            DOWN,
            DOWN,
            ENTER,
            RIGHT,
            DOWN,
            DOWN,
            DOWN,
            if translation { RIGHT } else { LEFT },
            UP,
            UP,
            UP,
            RIGHT,
            DOWN,
            if layout { RIGHT } else { LEFT },
            DOWN,
            if typing { RIGHT } else { LEFT },
            ENTER,
            ESC,
        ],
        data.as_ref(),
    );
}

fn has(screen: &str, label: &str) {
    assert!(screen.contains(label), "missing {label:?}\n{screen}");
}
fn result(screen: &str, perfect: bool) {
    has(screen, "WPM:");
    assert_eq!(screen.contains(PERFECT_ACCURACY), perfect, "{screen}");
    if perfect {
        let wpm: f64 = screen
            .split("WPM:")
            .nth(1)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert!(wpm > 0.0, "{screen}");
    }
}
fn case_value(screen: &str) -> &str {
    screen
        .split("Ignore case:")
        .nth(1)
        .unwrap()
        .lines()
        .find(|l| l.contains('<'))
        .unwrap()
        .trim()
}
fn inspect(data: &TestDataDir) -> TestRun {
    run_wpm_capture(
        &[
            DOWN, DOWN, DOWN, ENTER, RIGHT, RIGHT, RIGHT, LEFT, LEFT, LEFT, ESC, ESC,
        ],
        data.as_ref(),
    )
}

#[test]
fn settings_default_independent_saved_cancelled_and_absent_from_other_sections() {
    let data = fixture("AbC", "Кот", "Cat");
    let initial = inspect(&data);
    assert_eq!(case_value(&initial.screens[4]), "< Off >");
    assert_eq!(case_value(&initial.screens[5]), "< Off >");
    assert!(!initial.screens[3].contains("Ignore case"));
    has(&initial.screens[4], "Enter : save");
    for (typing, translation) in [(true, false), (false, true)] {
        configure(&data, typing, translation, false);
        let reopened = inspect(&data);
        assert_eq!(
            case_value(&reopened.screens[4]),
            if translation { "< On >" } else { "< Off >" }
        );
        assert_eq!(
            case_value(&reopened.screens[5]),
            if typing { "< On >" } else { "< Off >" }
        );
    }
    let run = run_wpm_capture(
        &[
            DOWN, DOWN, DOWN, ENTER, RIGHT, DOWN, DOWN, DOWN, LEFT, ESC, ENTER, RIGHT, ESC, ESC,
        ],
        data.as_ref(),
    );
    assert_eq!(case_value(&run.screens[11]), "< On >");
    // Cancel the Typing switch too.
    run_wpm_capture(
        &[
            DOWN, DOWN, DOWN, ENTER, RIGHT, RIGHT, DOWN, DOWN, RIGHT, ESC, ESC,
        ],
        data.as_ref(),
    );
    assert_eq!(case_value(&inspect(&data).screens[5]), "< Off >");
}

#[test]
fn old_and_partially_present_json_preserve_every_other_setting() {
    for fields in [
        json!({}),
        json!({"typing_ignore_case":true}),
        json!({"translation_ignore_case":true}),
    ] {
        let data = fixture("AbC", "Кот", "Cat");
        let mut settings = json!({"quick_start_mode":"typing","quick_start_time":90,
        "quick_start_wordset":"ru_5000","quick_start_translation_set":"ru_en_b1",
        "translation_set":"ru_es_b2","translation_time":120,"typing_layout_conversion":true,
        });
        settings
            .as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        data.write_settings(settings.clone());
        let run = inspect(&data);
        assert_eq!(
            case_value(&run.screens[4]),
            if fields["translation_ignore_case"] == true {
                "< On >"
            } else {
                "< Off >"
            }
        );
        assert_eq!(
            case_value(&run.screens[5]),
            if fields["typing_ignore_case"] == true {
                "< On >"
            } else {
                "< Off >"
            }
        );
        for label in ["ru_es_b2", "2 min"] {
            has(&run.screens[4], label);
        }
        assert_eq!(
            case_value(&run.screens[5]),
            if fields["typing_ignore_case"] == true {
                "< On >"
            } else {
                "< Off >"
            }
        );
        // Saving through UI must preserve every existing JSON value.
        run_wpm_capture(&[DOWN, DOWN, DOWN, ENTER, ENTER, ESC], data.as_ref());
        let saved: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(data.as_ref().join("settings.json")).unwrap())
                .unwrap();
        for (name, value) in settings.as_object().unwrap() {
            assert_eq!(&saved[name], value, "{name}");
        }
    }
}

#[test]
fn each_start_path_restart_and_opposite_flags_use_the_selected_mode_policy() {
    let data = fixture("AbC", "Кот", "Cat");
    for (typing, translation) in [(true, false), (false, true)] {
        configure(&data, typing, translation, false);
        for start in [vec![ENTER], vec![DOWN, DOWN, ENTER, ENTER]] {
            let mut keys = start;
            let at = keys.len();
            keys.extend([b"aBc", ENTER, b"aBc", ESC, ESC]);
            let run = run_wpm_capture(&keys, data.as_ref());
            result(&run.screens[at], typing);
            result(&run.screens[at + 2], typing);
        }
        for start in [vec![DOWN, ENTER], vec![DOWN, DOWN, ENTER, RIGHT, ENTER]] {
            let mut keys = start;
            let at = keys.len();
            keys.extend(["кОТ cAT".as_bytes(), ENTER, "кОТ cAT".as_bytes(), ESC, ESC]);
            let run = run_wpm_capture(&keys, data.as_ref());
            result(&run.screens[at], translation);
            result(&run.screens[at + 2], translation);
        }
        // Select Translation as Quick Start through the UI, then test it and Restart.
        run_wpm_capture(
            &[DOWN, DOWN, DOWN, ENTER, DOWN, RIGHT, ENTER, ESC],
            data.as_ref(),
        );
        let run = run_wpm_capture(
            &[
                ENTER,
                "кОТ cAT".as_bytes(),
                ENTER,
                "кОТ cAT".as_bytes(),
                ESC,
                ESC,
            ],
            data.as_ref(),
        );
        result(&run.screens[1], translation);
        result(&run.screens[3], translation);
        run_wpm_capture(
            &[DOWN, DOWN, DOWN, ENTER, DOWN, LEFT, ENTER, ESC],
            data.as_ref(),
        );
    }
}

#[test]
fn typing_layout_and_case_are_independent_and_off_restores_zero_stats() {
    let data = fixture("A", "Кот", "Cat");
    for layout in [false, true] {
        configure(&data, true, false, layout);
        for (input, correct) in [("a", true), ("ф", layout)] {
            let run = run_wpm_capture(&[ENTER, input.as_bytes(), ESC, ESC], data.as_ref());
            result(&run.screens[1], correct);
        }
    }
    configure(&data, false, true, false);
    let run = run_wpm_capture(&[ENTER, b"a", ESC, ESC], data.as_ref());
    has(&run.screens[1], "Accuracy: 0%");
    has(&run.screens[1], "WPM: 0");
}

#[test]
fn spanish_both_sides_phrases_multiple_pairs_and_canonical_text() {
    let data = fixture("AbC", "Кот", "Cat");
    data.seed_spanish(
        "ru_es_a1",
        &[
            ("Меня зовут Серхио", "Me llamo Sergio"),
            ("Меня зовут Серхио", "Me llamo Sergio"),
        ],
    );
    // Existing set selection is unrelated to the new flag.
    data.write_settings(json!({"translation_set":"ru_es_a1"}));
    configure(&data, false, true, false);
    let answer = "меня зовут серхио me llamo sergio";
    let run = run_wpm_capture(
        &[
            DOWN,
            ENTER,
            answer.as_bytes(),
            ENTER,
            answer.as_bytes(),
            ESC,
            ESC,
        ],
        data.as_ref(),
    );
    has(&run.screens[1], "Меня зовут Серхио Me llamo Sergio");
    result(&run.screens[4], true);
    configure(&data, false, false, false);
    let run = run_wpm_capture(
        &[
            DOWN,
            ENTER,
            answer.as_bytes(),
            ENTER,
            answer.as_bytes(),
            ESC,
            ESC,
        ],
        data.as_ref(),
    );
    result(&run.screens[4], false);
}

#[test]
fn spanish_variants_and_shift_punctuation_keep_language_boundaries() {
    for (ru, target, input, perfect, spanish) in [
        ("Я", "ÁÑÜ", "я anu", true, true),
        ("Я", "ÁÑÜ", "я фтг", true, true),
        ("Я", "ÁÑÜ", "я a\u{301}ñü", true, true),
        ("Я", "ÁÑÜ", "я enu", false, true),
        ("Ж", "x", "; X", true, false),
        ("ж", "x", ": X", true, false),
        ("Я", ";", "я Ж", true, false),
        ("Ж", "x", ": X", true, false),
        ("Я", ";", "я :", false, false),
        ("Я", "N", "я ñ", false, false),
        ("Я", "X", "я  x", false, false),
    ] {
        let data = fixture("AbC", ru, target);
        if spanish {
            data.seed_spanish("ru_es_a1", &[(ru, target)]);
            data.write_settings(json!({"translation_set":"ru_es_a1"}));
        }
        configure(&data, false, true, false);
        let run = run_wpm_capture(&[DOWN, ENTER, input.as_bytes(), ESC, ESC], data.as_ref());
        result(&run.screens[2], perfect);
    }
}

#[test]
fn terminal_cell_styles_agree_with_stats_and_backspace_in_both_modes() {
    use termwiz::color::ColorAttribute;
    let data = fixture("AbC", "АбВ", "Cat");
    configure(&data, true, true, false);
    for (start, canonical, correct) in [
        (vec![ENTER], "AbC", "a"),
        (vec![DOWN, ENTER], "АбВ Cat", "а"),
    ] {
        let mut keys = start;
        let at = keys.len();
        keys.extend([b"x", b"\x7f", correct.as_bytes(), ESC, ESC]);
        let run = run_wpm_capture(&keys, data.as_ref());
        let target = canonical.chars().next().unwrap().to_string();
        for (index, color, struck) in [(at, 1, true), (at + 2, 2, false)] {
            has(&run.screens[index], canonical);
            let attrs = &run.cells[index]
                .iter()
                .find(|(text, _)| text == &target)
                .unwrap()
                .1;
            assert_eq!(attrs.foreground(), ColorAttribute::PaletteIndex(color));
            assert_eq!(attrs.strikethrough(), struck);
        }
    }
}

#[test]
fn late_combining_mark_matches_joined_delivery_and_backspace_removes_whole_letter() {
    let data = fixture("CAFÉ", "Кот", "Cat");
    configure(&data, true, false, false);
    for separate in [false, true] {
        let keys: Vec<&[u8]> = if separate {
            vec![ENTER, b"cafe", "\u{301}".as_bytes(), ESC, ESC]
        } else {
            vec![ENTER, "cafe\u{301}".as_bytes(), ESC, ESC]
        };
        let run = run_wpm_capture_with_options(
            &keys,
            data.as_ref(),
            &[],
            &[(1, Duration::from_millis(200))],
        );
        let index = if separate { 2 } else { 1 };
        result(&run.screens[index], true);
        if separate {
            let time = |s: &str| {
                s.lines()
                    .find(|l| l.contains("Time:"))
                    .unwrap()
                    .trim()
                    .to_owned()
            };
            assert_eq!(time(&run.screens[1]), time(&run.screens[2]));
        }
    }
    let data = fixture("CAFÉX", "Кот", "Cat");
    configure(&data, true, false, false);
    let run = run_wpm_capture(
        &[
            ENTER,
            "cafe\u{301}".as_bytes(),
            b"\x7f",
            "éx".as_bytes(),
            ESC,
            ESC,
        ],
        data.as_ref(),
    );
    result(&run.screens[3], true);
}

#[test]
fn typing_accepts_direct_russian_and_spanish_case_with_either_layout_setting() {
    for (target, input) in [("АБВ", "абв"), ("ÁÑÜ", "áñü")] {
        let data = fixture(target, "Кот", "Cat");
        for layout in [false, true] {
            configure(&data, true, false, layout);
            let run = run_wpm_capture(&[ENTER, input.as_bytes(), ESC, ESC], data.as_ref());
            result(&run.screens[1], true);
        }
    }
}
