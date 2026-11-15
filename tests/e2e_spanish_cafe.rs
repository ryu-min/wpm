#![cfg(unix)]
mod common;
use common::{DOWN, ENTER, ESC, FakeAudio, RIGHT, TestDataDir, UP};
use std::collections::{BTreeMap, BTreeSet};
use unicode_normalization::UnicodeNormalization;
const SET: &str = "ru_es_cafe_beginner";
const F2: &[u8] = b"\x1bOQ";
const HINT: &str = "Type a word or phrase; case and accents are optional";

fn expected() -> BTreeMap<String, String> {
    // A separate requirements fixture: changes to delivered JSON must match the approved table.
    "| café | кофе |
| té | чай |
| agua | вода |
| leche | молоко |
| azúcar | сахар |
| pan | хлеб |
| un café | один кофе |
| un té | один чай |
| café con leche | кофе с молоком |
| sin azúcar | без сахара |
| por favor | пожалуйста |
| gracias | спасибо |
| una mesa | один столик |
| la cuenta | счёт |
| Quiero un café | Я хочу кофе |
| Hola | привет; здравствуйте |
| Me llamo Sergio | Меня зовут Серхио |
| Soy Sergio | Я Серхио |
| yo | я |
| trabajar | работать |
| Yo trabajo | Я работаю |
| estudiar | учиться |
| Yo estudio | Я учусь |
| ver | видеть |
| Yo veo | Я вижу |
| viajar | путешествовать |
| Yo viajo | Я путешествую |
| vivir | жить |
| Yo vivo | Я живу |
| Vivo en Rusia | Я живу в России |
| Vivo en España | Я живу в Испании |"
        .lines()
        .map(|line| {
            let fields: Vec<_> = line.trim_matches('|').split('|').map(str::trim).collect();
            (fields[0].to_string(), fields[1].to_string())
        })
        .collect()
}
fn fixture(pairs: &[(&str, &str)]) -> TestDataDir {
    let data = TestDataDir::new();
    data.seed_wordset("abc");
    data.seed_spanish(SET, pairs);
    data.write_settings(
        serde_json::json!({"dictation_set":SET,"quick_start_dictation_set":SET,
        "quick_start_mode":"dictation","translation_set":SET,"quick_start_translation_set":SET}),
    );
    data
}
fn check(screen: &str, labels: &[&str]) {
    for label in labels {
        assert!(screen.contains(label), "missing {label:?}\n{screen}");
    }
}
fn run(
    keys: &[&[u8]],
    data: &TestDataDir,
    audio: &FakeAudio,
    waits: &[(usize, &str)],
) -> common::TestRun {
    common::run_wpm_capture_waiting(
        keys,
        data.as_ref(),
        &[("WPM_TTS_BIN", &audio.binary)],
        &[],
        waits,
    )
}

#[test]
fn resource_matches_every_approved_pair_and_its_own_structural_contract() {
    let entries: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../resources/ru_es_cafe_beginner.json")).unwrap();
    assert_eq!(entries.len(), 31);
    let mut actual = BTreeMap::new();
    let mut ids = BTreeSet::new();
    for entry in entries {
        let es = entry["es"].as_str().unwrap();
        let ru = entry["ru"].as_str().unwrap();
        assert_eq!(entry["id"], format!("{SET}:{es}"));
        assert!(ids.insert(entry["id"].as_str().unwrap().to_string()));
        assert_eq!(es, es.nfc().collect::<String>());
        assert_eq!(ru, ru.nfc().collect::<String>());
        assert_eq!(es, es.trim());
        assert_eq!(ru, ru.trim());
        assert!(!es.contains("  "));
        assert!(
            es.chars()
                .all(|c| c.is_ascii_alphabetic() || "áéíóúüñÁÉÍÓÚÜÑ ".contains(c))
        );
        assert!(ru.chars().any(|c| ('А'..='я').contains(&c)));
        assert!(actual.insert(es.to_string(), ru.to_string()).is_none());
    }
    assert_eq!(actual, expected());
}

#[test]
fn binary_adds_cafe_once_on_fresh_and_legacy_installations_without_overwriting() {
    for legacy in [false, true] {
        let data = TestDataDir::new();
        if legacy {
            data.seed_wordset("personal words");
            data.seed_translation("мой кот", "my cat");
            data.seed_spanish("ru_es_a1", &[("мой дом", "casa")]);
            let conn = rusqlite::Connection::open(data.as_ref().join("wordset.db")).unwrap();
            conn.execute("UPDATE translation_sets SET name='ru_en_a2'", [])
                .unwrap();
            data.write_settings(serde_json::json!({"translation_set":"ru_en_a2","quick_start_translation_set":"ru_en_a2"}));
        }
        for _ in 0..2 {
            common::run_wpm_capture(&[ESC], data.as_ref());
        }
        let conn = rusqlite::Connection::open(data.as_ref().join("wordset.db")).unwrap();
        let (count, level): (i64, String) = conn
            .query_row(
                "SELECT count(*),level FROM vocabulary_sets WHERE name=?1",
                [SET],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((count, level), (1, "beginner".into()));
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM vocabulary_entries WHERE set_name=?1",
                [SET],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 31);
        let mut query = conn
            .prepare("SELECT es,ru FROM vocabulary_entries WHERE set_name=?1")
            .unwrap();
        let actual: BTreeMap<String, String> = query
            .query_map([SET], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(actual, expected());
        if legacy {
            let old: String = conn
                .query_row(
                    "SELECT words FROM word_sets WHERE name='en_1000'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(old, "personal words");
            let old: String = conn
                .query_row(
                    "SELECT pairs_json FROM translation_sets WHERE name='ru_en_a2'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&old).unwrap(),
                serde_json::json!([{"ru":"мой кот","en":"my cat"}])
            );
            let old: String = conn
                .query_row(
                    "SELECT ru FROM vocabulary_entries WHERE set_name='ru_es_a1'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(old, "мой дом");
            let ui =
                common::run_wpm_capture(&[DOWN, DOWN, DOWN, ENTER, RIGHT, ESC, ESC], data.as_ref());
            check(&ui.screens[4], &["< ru_en_a2 >"]);
        }
    }
}

#[test]
fn phrase_is_hidden_repeated_canonically_checked_once_and_keeps_policy_on_restart() {
    let data = fixture(&[("Меня зовут Серхио", "Me llamo Sergio")]);
    let audio = FakeAudio::new(&data, "success");
    let result = run(
        &[
            ENTER,
            F2,
            b"me llamo sergio",
            ENTER,
            ENTER,
            ENTER,
            b"ME LLAMO SERGIO",
            ENTER,
            ESC,
            ESC,
        ],
        &data,
        &audio,
        &[(0, "Audio ready"), (1, "Audio ready"), (5, "Audio ready")],
    );
    for index in [0, 1, 2] {
        check(&result.screens[index], &[HINT, "Word 1 / 1", "Checked: 0"]);
        for secret in [
            "Me llamo Sergio",
            "Меня зовут Серхио",
            "Answer:",
            "Translation:",
        ] {
            assert!(!result.screens[index].contains(secret));
        }
    }
    for index in [3, 7] {
        check(
            &result.screens[index],
            &[
                "Answer: Me llamo Sergio",
                "Translation: Меня зовут Серхио",
                "Checked: 1  Correct: 1  Errors: 0",
            ],
        );
    }
    check(
        &result.screens[4],
        &["Dictation result", "Checked: 1", "Correct words: 100%"],
    );
    check(&result.screens[8], &["Quick Start"]);
    let starts: Vec<_> = audio
        .events()
        .into_iter()
        .filter(|e| e["event"] == "start")
        .collect();
    assert_eq!(starts.len(), 3);
    assert!(starts.iter().all(|e| e["word"] == "Me llamo Sergio"));
}

#[test]
fn cafe_checks_whole_phrases_spaces_layout_and_optional_accents() {
    for (target, answer, accepted) in [
        ("Vivo en España", "vivo en espana", true),
        ("Vivo en España", "мшмщ ут уызфтф", true),
        ("Vivo en España", "  vivo en espana  ", true),
        ("Vivo en España", "vivo  en espana", false),
        ("Vivo en España", "vivo en", false),
        ("Me llamo Sergio", "me llamosergio", false),
        ("Yo trabajo", "trabajo", false),
        ("un café", "cafe", false),
    ] {
        let data = fixture(&[("перевод", target)]);
        let audio = FakeAudio::new(&data, "success");
        let result = run(
            &[ENTER, answer.as_bytes(), ENTER, ENTER, ESC, ESC],
            &data,
            &audio,
            &[(0, "Audio ready")],
        );
        check(
            &result.screens[2],
            &[&format!("Answer: {target}"), "Checked: 1"],
        );
        check(
            &result.screens[3],
            &[
                "Checked: 1",
                if accepted { "Correct: 1" } else { "Errors: 1" },
            ],
        );
    }
    let data = fixture(&[("привет", "hola")]);
    data.seed_spanish("ru_es_a1", &[("привет", "hola")]);
    data.write_settings(serde_json::json!({"quick_start_mode":"dictation"}));
    let audio = FakeAudio::new(&data, "success");
    let result = run(
        &[ENTER, b"HOLA", ENTER, ESC, ESC],
        &data,
        &audio,
        &[(0, "Audio ready")],
    );
    check(
        &result.screens[0],
        &["Type lowercase; accents are optional"],
    );
    check(&result.screens[2], &["Incorrect", "Errors: 1"]);
}

#[test]
fn translation_types_complete_pairs_without_audio_and_retains_case() {
    // Both possible shuffled orders must produce 100% with a newline between full pairs.
    let pairs = [
        ("Меня зовут Серхио", "Me llamo Sergio"),
        ("Я живу в Испании", "Vivo en España"),
    ];
    let data = fixture(&pairs);
    let missing = data.as_ref().join("missing-audio");
    fn full_pairs(screen: &str) -> Vec<u8> {
        check(screen, &["Me llamo Sergio", "Vivo en España"]);
        let name = "Меня зовут Серхио Me llamo Sergio";
        let country = "Я живу в Испании Vivo en Espana";
        if screen.find("Me llamo Sergio").unwrap() < screen.find("Vivo en España").unwrap() {
            format!("{name}\r{country}").into_bytes()
        } else {
            format!("{country}\r{name}").into_bytes()
        }
    }
    let mixed = common::run_wpm_capture_adaptive(
        &[DOWN, ENTER, b"", ESC, ESC],
        data.as_ref(),
        &[("WPM_TTS_BIN", &missing)],
        &[],
        &[],
        &[(2, full_pairs)],
    );
    check(&mixed.screens[2], &[common::PERFECT_ACCURACY]);
    // Repeated identical full pairs make each order deterministic while still exercising both phrases.
    for (ru, es) in pairs {
        let data = fixture(&[(ru, es), (ru, es)]);
        let answer = format!("{ru} {}", es.replace('ñ', "n"));
        let missing = data.as_ref().join("missing-audio");
        let result = common::run_wpm_capture_with_options(
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
            &[("WPM_TTS_BIN", &missing)],
            &[],
        );
        check(&result.screens[4], &[common::PERFECT_ACCURACY]);
        let wrong = format!("{ru} {}", es.to_lowercase());
        let result = common::run_wpm_capture(
            &[
                DOWN,
                ENTER,
                wrong.as_bytes(),
                ENTER,
                wrong.as_bytes(),
                ESC,
                ESC,
            ],
            data.as_ref(),
        );
        // Each phrase loses two uppercase letters, hence four errors across
        // both pairs: 63/67 or 59/63 correct characters, both rounded to 94%.
        check(&result.screens[4], &["> Restart", "Menu", "Accuracy: 94%"]);
    }
}

#[test]
fn real_embedded_set_yields_all_31_distinct_canonical_tasks_with_length_50() {
    let data = TestDataDir::new();
    data.write_settings(serde_json::json!({"quick_start_mode":"dictation","quick_start_dictation_set":SET,"quick_start_dictation_length":50}));
    let audio = FakeAudio::new(&data, "success");
    let mut keys = vec![ENTER];
    let mut waits = vec![(0, "Audio ready")];
    for index in 0..31 {
        keys.extend([b"wrong".as_slice(), ENTER, ENTER]);
        if index < 30 {
            waits.push((keys.len() - 1, "Audio ready"));
        }
    }
    keys.extend([ESC, ESC]);
    let result = run(&keys, &data, &audio, &waits);
    let expected = expected();
    let mut revealed = BTreeSet::new();
    let spoken: Vec<_> = audio
        .events()
        .into_iter()
        .filter(|e| e["event"] == "start")
        .map(|e| e["word"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(spoken.len(), 31);
    for (index, es) in spoken.iter().enumerate() {
        assert!(revealed.insert(es.clone()));
        check(
            &result.screens[index * 3 + 2],
            &[
                &format!("Answer: {es}"),
                &format!("Translation: {}", expected[es]),
            ],
        );
    }
    assert_eq!(revealed, expected.into_keys().collect());
    check(
        &result.screens[93],
        &["Dictation result", "Checked: 31", "Errors: 31"],
    );
}

#[test]
fn settings_and_select_mode_choose_cafe_for_both_modes_and_preserve_independent_settings() {
    for dictation in [false, true] {
        let data = fixture(&[("Меня зовут Серхио", "Me llamo Sergio")]);
        data.write_settings(serde_json::json!({}));
        let audio = FakeAudio::new(&data, "success");
        let mut keys = vec![DOWN, DOWN, DOWN, ENTER, RIGHT];
        if dictation {
            keys.extend([RIGHT, RIGHT]);
        }
        keys.push(DOWN);
        keys.extend(std::iter::repeat_n(RIGHT, 30));
        let chosen = keys.len() - 1;
        keys.extend([ENTER, ESC]);
        let save = common::run_wpm_capture(&keys, data.as_ref());
        check(&save.screens[chosen], &[&format!("< {SET} >")]);
        // Reopen from disk: the direct selection changed, Quick Start is still the default.
        let reopen = common::run_wpm_capture(&[DOWN, DOWN, DOWN, ENTER, ESC, ESC], data.as_ref());
        check(&reopen.screens[3], &["< typing >"]);
        let settings: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(data.as_ref().join("settings.json")).unwrap(),
        )
        .unwrap();
        let direct_key = if dictation {
            "dictation_set"
        } else {
            "translation_set"
        };
        let quick_key = if dictation {
            "quick_start_dictation_set"
        } else {
            "quick_start_translation_set"
        };
        assert_eq!(settings[direct_key], SET);
        assert_ne!(settings[quick_key], SET);
        let mut keys = vec![DOWN, DOWN, DOWN, ENTER, DOWN, RIGHT];
        if dictation {
            keys.push(RIGHT);
        }
        keys.push(DOWN);
        keys.extend(std::iter::repeat_n(RIGHT, 30));
        keys.extend([ENTER, ESC]);
        common::run_wpm_capture(&keys, data.as_ref());
        let reopen = common::run_wpm_capture(&[DOWN, DOWN, DOWN, ENTER, ESC, ESC], data.as_ref());
        check(
            &reopen.screens[3],
            &[
                &format!("< {SET} >"),
                if dictation {
                    "< dictation >"
                } else {
                    "< translation >"
                },
            ],
        );
        let settings: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(data.as_ref().join("settings.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(settings[direct_key], SET);
        assert_eq!(settings[quick_key], SET);
        let mut starts = vec![
            vec![ENTER],
            if dictation {
                vec![UP, UP, ENTER]
            } else {
                vec![DOWN, ENTER]
            },
        ];
        let mut select = vec![DOWN, DOWN, ENTER, RIGHT];
        if dictation {
            select.push(RIGHT);
        }
        select.push(DOWN);
        select.extend(std::iter::repeat_n(RIGHT, 30));
        let selected = select.len() - 1;
        select.push(ENTER);
        starts.push(select);
        for (path, mut keys) in starts.into_iter().enumerate() {
            let start = keys.len() - 1;
            let result_index;
            if dictation {
                keys.extend([b"me llamo sergio".as_slice(), ENTER]);
                result_index = keys.len() - 1;
            } else {
                keys.push("Меня зовут Серхио Me llamo Sergio".as_bytes());
                result_index = keys.len() - 1;
            }
            keys.extend([ESC, ESC]);
            let audio_waits = [(start, "Audio ready")];
            let result = run(
                &keys,
                &data,
                &audio,
                if dictation { &audio_waits } else { &[] },
            );
            if path == 2 {
                check(&result.screens[selected], &[&format!("< {SET} >")]);
            }
            check(
                &result.screens[result_index],
                if dictation {
                    &["Answer: Me llamo Sergio", "Correct: 1"]
                } else {
                    &[common::PERFECT_ACCURACY]
                },
            );
        }
    }
}
