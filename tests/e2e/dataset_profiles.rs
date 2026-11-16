use crate::common::{DOWN, ENTER, ESC, FakeAudio, RIGHT, TestDataDir, UP};
use std::collections::{BTreeMap, BTreeSet};
#[allow(dead_code)]
#[path = "../../src/dataset/mod.rs"]
mod dataset;
use dataset::{Dataset, Profile};
const F2: &[u8] = b"\x1bOQ";
const HINT: &str = "Type a word or phrase; case and accents are optional";
fn descriptors() -> Vec<Dataset> {
    let mut paths: Vec<_> = std::fs::read_dir("resources/datasets")
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|p| {
            Dataset::parse(p.to_str().unwrap(), &std::fs::read_to_string(&p).unwrap()).unwrap()
        })
        .collect()
}
fn phrase_id() -> String {
    descriptors()
        .into_iter()
        .find(|d| d.profile == Profile::SpanishPhrases)
        .unwrap()
        .id
}
fn fixture(pairs: &[(&str, &str)]) -> TestDataDir {
    let data = TestDataDir::new();
    let set = phrase_id();
    data.seed_wordset("abc");
    data.seed_spanish(&set, pairs);
    data.write_settings(
        serde_json::json!({"dictation_set":set,"quick_start_dictation_set":set,
        "quick_start_mode":"dictation","translation_set":set,"quick_start_translation_set":set}),
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
) -> crate::common::TestRun {
    crate::common::run_wpm_capture_waiting(
        keys,
        data.as_ref(),
        &[("WPM_TTS_BIN", &audio.binary)],
        &[],
        waits,
    )
}

#[test]
fn every_discovered_descriptor_is_seeded_once_and_preserves_existing_data() {
    for d in descriptors() {
        let data = TestDataDir::new();
        data.seed_wordset("personal words");
        data.seed_translation("мой кот", "my cat");
        let previous = rusqlite::Connection::open(data.as_ref().join("wordset.db")).unwrap();
        previous
            .execute(
                "UPDATE word_sets SET name='personal_words' WHERE name='en_1000'",
                [],
            )
            .unwrap();
        previous
            .execute(
                "UPDATE translation_sets SET name='personal_translation' WHERE name='ru_en_a1'",
                [],
            )
            .unwrap();
        drop(previous);
        data.write_settings(serde_json::json!({"translation_set":"ru_en_a2"}));
        for _ in 0..2 {
            crate::common::run_wpm_capture(&[ESC], data.as_ref());
        }
        let conn = rusqlite::Connection::open(data.as_ref().join("wordset.db")).unwrap();
        if d.profile.typing() {
            let (count, words): (i64, String) = conn
                .query_row(
                    "SELECT count(*),words FROM word_sets WHERE name=?1",
                    [&d.id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .unwrap();
            assert_eq!(count, 1);
            assert_eq!(words, d.words().join(" "));
        } else if d.profile.spanish() {
            let mut query = conn
                .prepare("SELECT id,ru,es FROM vocabulary_entries WHERE set_name=?1 ORDER BY id")
                .unwrap();
            let actual: Vec<(String, String, String)> = query
                .query_map([&d.id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .unwrap()
                .map(Result::unwrap)
                .collect();
            let mut expected: Vec<_> = d
                .pairs()
                .into_iter()
                .map(|p| (p.id, p.ru, p.target))
                .collect();
            expected.sort();
            assert_eq!(actual, expected);
        } else {
            let raw: String = conn
                .query_row(
                    "SELECT pairs_json FROM translation_sets WHERE name=?1",
                    [&d.id],
                    |r| r.get(0),
                )
                .unwrap();
            let expected: Vec<_> = d
                .pairs()
                .into_iter()
                .map(|p| serde_json::json!({"ru":p.ru,"en":p.target}))
                .collect();
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&raw).unwrap(),
                serde_json::json!(expected)
            );
        }
        let words: String = conn
            .query_row(
                "SELECT words FROM word_sets WHERE name='personal_words'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(words, "personal words");
        let settings: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(data.as_ref().join("settings.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(settings["translation_set"], "ru_en_a2");
        descriptor_mode_availability(&data, &d);
        // Preserve a locally edited built-in and its selected id on two more startups.
        if d.profile.typing() {
            conn.execute(
                "UPDATE word_sets SET words='personal' WHERE name=?1",
                [&d.id],
            )
            .unwrap();
        } else if d.profile.spanish() {
            conn.execute(
                "UPDATE vocabulary_entries SET ru='локальный перевод' WHERE set_name=?1",
                [&d.id],
            )
            .unwrap();
        } else {
            conn.execute("UPDATE translation_sets SET pairs_json='[{\"ru\":\"мой кот\",\"en\":\"mine\"}]' WHERE name=?1",[&d.id]).unwrap();
        }
        let stored = || {
            let mut query=conn.prepare("SELECT words FROM word_sets WHERE name=?1 UNION ALL SELECT pairs_json FROM translation_sets WHERE name=?1 UNION ALL SELECT id||char(9)||ru||char(9)||es FROM vocabulary_entries WHERE set_name=?1 ORDER BY 1").unwrap();
            query
                .query_map([&d.id], |r| r.get::<_, String>(0))
                .unwrap()
                .map(Result::unwrap)
                .collect::<Vec<_>>()
        };
        let before = stored();
        data.write_settings(serde_json::json!({"translation_set":d.id,"dictation_set":d.id,"quick_start_wordset":d.id}));
        for _ in 0..2 {
            crate::common::run_wpm_capture(&[ESC], data.as_ref());
        }
        assert_eq!(stored(), before);
        let saved: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(data.as_ref().join("settings.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(saved["translation_set"], d.id);
    }
}

fn descriptor_geometry(d: &Dataset) -> (usize, usize) {
    let width = |text: &str| termwiz::cell::unicode_column_width(text, None);
    let entry_width = if d.profile.typing() {
        d.words().iter().map(|w| width(w)).max().unwrap()
    } else {
        d.pairs()
            .iter()
            .map(|p| width(&p.ru) + width(&p.target) + 20)
            .max()
            .unwrap()
    };
    // Typing/Translation uses 70% of the terminal for text. Leave enough space
    // for an entire pair, the Dictation labels and full ids in both selectors.
    ((entry_width.max(width(&d.id) + 10) * 2).max(80), 24)
}
fn run_descriptor(
    keys: &[&[u8]],
    data: &TestDataDir,
    d: &Dataset,
    audio: &FakeAudio,
    waits: &[(usize, &str)],
) -> crate::common::TestRun {
    crate::common::run_wpm_capture_geometry(
        keys,
        data.as_ref(),
        &[("WPM_TTS_BIN", &audio.binary)],
        &[],
        waits,
        &[],
        descriptor_geometry(d),
    )
}
fn descriptor_mode_availability(data: &TestDataDir, d: &Dataset) {
    let modes: Vec<_> = if d.profile.typing() {
        vec!["typing"]
    } else if d.profile.spanish() {
        vec!["translation", "dictation"]
    } else {
        vec!["translation"]
    };
    for mode in modes {
        data.write_settings(serde_json::json!({"quick_start_mode":mode,"quick_start_wordset":d.id,"quick_start_translation_set":d.id,"quick_start_dictation_set":d.id,"translation_set":d.id,"dictation_set":d.id}));
        let audio = FakeAudio::new(data, "success");
        let settings = run_descriptor(&[DOWN, DOWN, DOWN, ENTER, ESC, ESC], data, d, &audio, &[]);
        check(&settings.screens[3], &[&format!("< {} >", d.id)]);
        let mut keys = vec![DOWN, DOWN, ENTER];
        if mode != "typing" {
            keys.push(RIGHT);
            if mode == "dictation" {
                keys.push(RIGHT);
            }
        } else {
            let list = options(data, d.profile);
            keys.push(DOWN);
            navigate(&mut keys, &list[0], &d.id, &list);
        }
        let chosen = keys.len() - 1;
        keys.push(ENTER);
        let started = keys.len() - 1;
        keys.extend([ESC, ESC]);
        let waits = [(
            started,
            if mode == "dictation" {
                "Audio ready"
            } else {
                "0 wpm"
            },
        )];
        let result = run_descriptor(&keys, data, d, &audio, &waits);
        check(&result.screens[chosen], &[&format!("< {} >", d.id)]);
        assert!(!result.screens[started].contains("unavailable or damaged"));
        if mode == "dictation" {
            check(&result.screens[started], &["Word 1 /"]);
        } else {
            assert!(
                d.entries
                    .iter()
                    .any(|v| result.screens[started].contains(if d.profile.typing() {
                        v.as_str().unwrap()
                    } else {
                        v["target"].as_str().unwrap()
                    })),
                "{} ({mode}): {}",
                d.id,
                result.screens[started]
            );
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
fn spanish_profiles_check_whole_phrases_spaces_layout_and_optional_accents() {
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
    let mixed = crate::common::run_wpm_capture_adaptive(
        &[DOWN, ENTER, b"", ESC, ESC],
        data.as_ref(),
        &[("WPM_TTS_BIN", &missing)],
        &[],
        &[],
        &[(2, full_pairs)],
    );
    check(&mixed.screens[2], &[crate::common::PERFECT_ACCURACY]);
    // Repeated identical full pairs make each order deterministic while still exercising both phrases.
    for (ru, es) in pairs {
        let data = fixture(&[(ru, es), (ru, es)]);
        let answer = format!("{ru} {}", es.replace('ñ', "n"));
        let missing = data.as_ref().join("missing-audio");
        let result = crate::common::run_wpm_capture_with_options(
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
        check(&result.screens[4], &[crate::common::PERFECT_ACCURACY]);
        let wrong = format!("{ru} {}", es.to_lowercase());
        let result = crate::common::run_wpm_capture(
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
fn embedded_spanish_catalogs_count_entries_reveal_canonical_pairs_and_do_not_repeat() {
    for d in descriptors().into_iter().filter(|d| d.profile.spanish()) {
        let data = TestDataDir::new();
        assert_spanish_catalog(&d, &data);
    }
}
fn assert_spanish_catalog(d: &Dataset, data: &TestDataDir) {
    let length = d.entries.len().min(50);
    data.write_settings(serde_json::json!({"quick_start_mode":"dictation","quick_start_dictation_set":d.id,"quick_start_dictation_length":50}));
    let audio = FakeAudio::new(data, "success");
    let mut keys = vec![ENTER];
    let mut waits = vec![(0, "Audio ready")];
    for index in 0..length {
        keys.extend([b"wrong".as_slice(), ENTER, ENTER]);
        if index + 1 < length {
            waits.push((keys.len() - 1, "Audio ready"));
        }
    }
    keys.extend([ESC, ESC]);
    let result = run_descriptor(&keys, data, d, &audio, &waits);
    let expected: BTreeMap<_, _> = d.pairs().into_iter().map(|p| (p.target, p.ru)).collect();
    let spoken: Vec<_> = audio
        .events()
        .into_iter()
        .filter(|e| e["event"] == "start")
        .map(|e| e["word"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(spoken.len(), length);
    assert_eq!(spoken.iter().collect::<BTreeSet<_>>().len(), length);
    for (index, es) in spoken.iter().enumerate() {
        check(
            &result.screens[index * 3 + 2],
            &[
                &format!("Answer: {es}"),
                &format!("Translation: {}", expected[es]),
            ],
        );
    }
    check(
        &result.screens[length * 3],
        &["Dictation result", &format!("Checked: {length}")],
    );
}

#[test]
fn descriptor_checks_handle_long_ids_targets_and_russian_translations() {
    let id = format!("long_{}end", "dataset_".repeat(16));
    let target = "Quisiera pedir un cafe con leche sin azucar y dos vasos de agua para mis amigos";
    let ru = "Я хотел бы заказать кофе с молоком без сахара и два стакана воды для моих друзей за соседним столиком";
    let raw=serde_json::json!({"schema_version":1,"id":id,"description":"Long data regression","profile":"spanish-phrases","entries":[{"id":format!("{id}:one"),"ru":ru,"target":target}]}).to_string();
    let descriptor = Dataset::parse(&format!("{id}.json"), &raw).unwrap();
    let data = TestDataDir::new();
    data.seed_spanish(&id, &[(ru, target)]);
    crate::common::run_wpm_capture(&[ESC], data.as_ref());
    descriptor_mode_availability(&data, &descriptor);
    let catalog_data = TestDataDir::new();
    catalog_data.seed_spanish(&id, &[(ru, target)]);
    assert_spanish_catalog(&descriptor, &catalog_data);
}

fn options(data: &TestDataDir, profile: Profile) -> Vec<String> {
    let conn = rusqlite::Connection::open(data.as_ref().join("wordset.db")).unwrap();
    let sql = if profile.typing() {
        "SELECT name FROM word_sets ORDER BY language,word_count"
    } else if profile.spanish() {
        "SELECT name FROM vocabulary_sets WHERE source_lang='ru' AND target_lang='es' ORDER BY name"
    } else {
        "SELECT name FROM translation_sets UNION SELECT name FROM vocabulary_sets ORDER BY name"
    };
    let mut stmt = conn.prepare(sql).unwrap();
    stmt.query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}
fn navigate<'a>(keys: &mut Vec<&'a [u8]>, from: &str, to: &str, options: &[String]) {
    let from = if let Some(index) = options.iter().position(|x| x == from) {
        index
    } else {
        keys.push(crate::common::LEFT);
        0
    };
    let to = options.iter().position(|x| x == to).unwrap();
    keys.extend(std::iter::repeat_n(
        if to >= from {
            RIGHT
        } else {
            crate::common::LEFT
        },
        to.abs_diff(from),
    ));
}
#[test]
fn five_profiles_route_settings_select_mode_quick_start_and_persist() {
    for profile in [
        Profile::TypingRu,
        Profile::TypingEn,
        Profile::TranslationRuEn,
        Profile::SpanishWords,
        Profile::SpanishPhrases,
    ] {
        let data = TestDataDir::new();
        let id = if profile == Profile::SpanishPhrases {
            phrase_id()
        } else {
            "practice_without_language_prefix".into()
        };
        let geometry = (
            (termwiz::cell::unicode_column_width(&id, None) + 10) * 2,
            24,
        );
        let capture = |keys: &[&[u8]]| {
            crate::common::run_wpm_capture_geometry(
                keys,
                data.as_ref(),
                &[],
                &[],
                &[],
                &[],
                geometry,
            )
        };
        if profile.typing() {
            data.seed_wordset(if profile == Profile::TypingRu {
                "кот"
            } else {
                "cat"
            });
            let conn = rusqlite::Connection::open(data.as_ref().join("wordset.db")).unwrap();
            conn.execute(
                "UPDATE word_sets SET name=?1,language=?2",
                (
                    &id,
                    if profile == Profile::TypingRu {
                        "ru"
                    } else {
                        "en"
                    },
                ),
            )
            .unwrap();
        } else if profile.spanish() {
            data.seed_spanish(
                &id,
                &[(
                    "привет",
                    if profile == Profile::SpanishWords {
                        "hola"
                    } else {
                        "Hola"
                    },
                )],
            );
        } else {
            data.seed_translation("кот", "cat");
            let conn = rusqlite::Connection::open(data.as_ref().join("wordset.db")).unwrap();
            conn.execute("UPDATE translation_sets SET name=?1", [&id])
                .unwrap();
        }
        data.write_settings(serde_json::json!({}));
        capture(&[ESC]);
        let list = options(&data, profile);
        if !profile.typing() {
            let mut keys = vec![DOWN, DOWN, DOWN, ENTER, RIGHT];
            if profile.spanish() {
                keys.extend([RIGHT, RIGHT]);
            }
            keys.push(DOWN);
            navigate(
                &mut keys,
                if profile.spanish() {
                    "ru_es_a1"
                } else {
                    "ru_en_a1"
                },
                &id,
                &list,
            );
            let chosen = keys.len() - 1;
            keys.extend([ENTER, ESC]);
            let save = capture(&keys);
            check(&save.screens[chosen], &[&format!("< {id} >")]);
        }
        let mut keys = vec![DOWN, DOWN, DOWN, ENTER, DOWN];
        if !profile.typing() {
            keys.push(RIGHT);
            if profile.spanish() {
                keys.push(RIGHT);
            }
        }
        keys.push(DOWN);
        navigate(
            &mut keys,
            if profile.typing() {
                "en_1000"
            } else if profile.spanish() {
                "ru_es_a1"
            } else {
                "ru_en_a1"
            },
            &id,
            &list,
        );
        keys.extend([ENTER, ESC]);
        capture(&keys);
        let reopen = capture(&[DOWN, DOWN, DOWN, ENTER, ESC, ESC]);
        check(&reopen.screens[3], &[&format!("< {id} >")]);
        let audio = FakeAudio::new(&data, "success");
        let answer = if profile == Profile::TypingRu {
            "кот"
        } else if profile == Profile::TypingEn {
            "cat"
        } else if profile == Profile::TranslationRuEn {
            "кот cat"
        } else {
            "hola"
        };
        let mut paths = vec![vec![ENTER]];
        let mut select = vec![DOWN, DOWN, ENTER];
        if !profile.typing() {
            select.push(RIGHT);
            if profile.spanish() {
                select.push(RIGHT);
            }
        }
        if profile.typing() {
            select.push(DOWN);
            navigate(&mut select, &list[0], &id, &list);
        }
        let chosen = select.len() - 1;
        select.push(ENTER);
        paths.push(select);
        if !profile.typing() {
            paths.push(if profile.spanish() {
                vec![UP, UP, ENTER]
            } else {
                vec![DOWN, ENTER]
            });
        }
        for (path, mut keys) in paths.into_iter().enumerate() {
            let start = keys.len() - 1;
            keys.push(answer.as_bytes());
            if profile.spanish() {
                keys.push(ENTER);
            }
            let result_index = keys.len() - 1;
            keys.extend([ESC, ESC]);
            let wait = [(start, "Audio ready")];
            let result = crate::common::run_wpm_capture_geometry(
                &keys,
                data.as_ref(),
                &[("WPM_TTS_BIN", &audio.binary)],
                &[],
                if profile.spanish() { &wait } else { &[] },
                &[],
                geometry,
            );
            if path == 1 {
                check(&result.screens[chosen], &[&format!("< {id} >")]);
            }
            check(
                &result.screens[result_index],
                if profile.spanish() {
                    &["Correct: 1"]
                } else {
                    &[crate::common::PERFECT_ACCURACY]
                },
            );
        }
        // A non-Spanish profile must never appear in the Dictation selector.
        if !profile.spanish() {
            assert!(!options(&data, Profile::SpanishWords).contains(&id));
        }
    }
}
