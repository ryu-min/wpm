use super::common::{DOWN, ENTER, ESC, LEFT, RIGHT, TestDataDir, UP, run_course_capture};
use serde_json::Value;

fn catalog() -> Value {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/spanish-course.json"
    )))
    .unwrap()
}
fn count() -> usize {
    catalog()["modules"][0]["topics"][0]["entries"]
        .as_array()
        .unwrap()
        .len()
}
fn answer(screen: &str) -> Vec<u8> {
    let catalog = catalog();
    for entry in catalog["modules"][0]["topics"][0]["entries"]
        .as_array()
        .unwrap()
    {
        if screen
            .lines()
            .any(|line| line.trim() == entry["ru"].as_str().unwrap())
        {
            return entry["target"].as_str().unwrap().as_bytes().to_vec();
        }
    }
    panic!("cannot identify question from screen: {screen}");
}
fn run(keys: &[&[u8]], data: &TestDataDir) -> super::common::TestRun {
    run_course_capture(keys, data.as_ref(), (80, 24), &[])
}

#[test]
fn centered_main_form_remembers_all_choices_and_module_topics() {
    for width in [80, 81, 120] {
        let data = TestDataDir::new();
        let result = run_course_capture(
            &[b"", RIGHT, DOWN, RIGHT, DOWN, RIGHT, ESC],
            data.as_ref(),
            (width, 30),
            &[],
        );
        let screen = &result.screens[0];
        for label in ["Испанский", "Модуль", "Раздел", "Режим"] {
            assert!(screen.contains(label), "{screen}");
        }
        for line in screen.lines().filter(|line| {
            line.contains('<')
                || ["Испанский", "Модуль:", "Раздел:", "Режим:"].contains(&line.trim())
        }) {
            let start = line.chars().take_while(|c| c.is_whitespace()).count();
            let len = line.trim().chars().count();
            assert!(
                (2 * start + len).abs_diff(width) <= 1,
                "uncentered {line:?}"
            );
        }
        let restart = run(&[b"", ESC], &data);
        let selected = &result.screens[5];
        for value in selected
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with('<'))
        {
            assert!(restart.screens[0].contains(value), "{value}");
        }
    }
}

fn training_text() -> String {
    catalog()["modules"][0]["topics"][0]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            format!(
                "{} {}",
                e["ru"].as_str().unwrap(),
                e["target"].as_str().unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
fn text_keys(text: &str) -> Vec<Vec<u8>> {
    let mut keys = Vec::new();
    for (index, line) in text.split('\n').enumerate() {
        if index > 0 {
            keys.push(ENTER.to_vec());
        }
        keys.push(line.as_bytes().to_vec());
    }
    keys
}

#[test]
fn training_types_both_sides_and_backspace_crosses_pair_boundary_with_cell_styles() {
    use termwiz::{cell::Underline, color::ColorAttribute};
    let data = TestDataDir::new();
    let result = super::common::run_course_observing(
        &[
            ENTER,
            b"x",
            b"\x7f",
            "в".as_bytes(),
            "ода agua".as_bytes(),
            ENTER,
            b"\x7f",
            ENTER,
            "кофе cafe".as_bytes(),
            ESC,
            ESC,
        ],
        data.as_ref(),
        &[],
        &[
            (1, |_, cells| {
                cells.iter().any(|(t, a)| t == "в" && a.strikethrough())
            }),
            (2, |_, cells| {
                cells
                    .iter()
                    .any(|(t, a)| t == "в" && a.underline() != Underline::None)
            }),
            (3, |_, cells| {
                cells
                    .iter()
                    .any(|(t, a)| t == "в" && a.foreground() == ColorAttribute::PaletteIndex(2))
            }),
            (5, |_, cells| {
                cells
                    .iter()
                    .any(|(t, a)| t == "к" && a.underline() != Underline::None)
            }),
            (6, |_, cells| {
                !cells
                    .iter()
                    .any(|(t, a)| t == "к" && a.underline() != Underline::None)
            }),
            (7, |_, cells| {
                cells
                    .iter()
                    .any(|(t, a)| t == "к" && a.underline() != Underline::None)
            }),
            (9, |screen, _| screen.contains("Модуль")),
        ],
    );
    assert!(
        result.screens[0].contains("вода agua"),
        "{}",
        result.screens[0]
    );
    for (index, color, struck) in [(1, 1, true), (3, 2, false)] {
        let attrs = &result.cells[index]
            .iter()
            .find(|(t, a)| {
                t == "в"
                    && a.strikethrough() == struck
                    && a.foreground() == ColorAttribute::PaletteIndex(color)
            })
            .unwrap()
            .1;
        assert_eq!(attrs.foreground(), ColorAttribute::PaletteIndex(color));
        assert_eq!(attrs.strikethrough(), struck);
    }
    let cursor = result.cells[2]
        .iter()
        .find(|(t, a)| t == "в" && a.foreground() == ColorAttribute::PaletteIndex(3))
        .unwrap();
    assert_ne!(cursor.1.underline(), Underline::None);
    // Cursor underline on the second pair disappears when Backspace removes its preceding newline.
    let second = |index: usize| {
        result.cells[index]
            .iter()
            .find(|(t, a)| t == "к" && a.underline() != Underline::None)
            .is_some()
    };
    assert!(second(5));
    assert!(!second(6));
    assert!(second(7));
    assert!(result.screens[9].contains("Модуль"));
}

#[test]
fn full_training_auto_finishes_with_metrics_and_restarts_same_text() {
    for wrong in [false, true] {
        let data = TestDataDir::new();
        let text = training_text();
        let text = if wrong {
            format!("X{}", &text["в".len()..])
        } else {
            text
        };
        let chunks = text_keys(&text);
        let mut keys = vec![ENTER];
        keys.extend(chunks.iter().map(Vec::as_slice));
        let finish = keys.len() - 1;
        keys.extend([ENTER, ESC, ESC]);
        let result = run(&keys, &data);
        let screen = &result.screens[finish];
        assert!(screen.contains("WPM"), "{screen}");
        assert!(screen.contains("Точность печати"), "{screen}");
        assert_eq!(screen.contains("100.0%"), !wrong, "{screen}");
        assert!(result.screens[finish + 1].contains("вода agua"));
    }
}

#[test]
fn exam_hides_answers_accepts_empty_and_offers_error_training() {
    let data = TestDataDir::new();
    let mut keys = vec![DOWN, DOWN, RIGHT, ENTER];
    keys.extend(std::iter::repeat_n(ENTER, count()));
    keys.extend([DOWN, ENTER, ESC, ESC]);
    let result = run(&keys, &data);
    assert!(result.screens[3].contains("Ваш ответ"));
    assert!(!result.screens[3].contains("По-испански"));
    assert!(
        result.screens[3 + count()].contains("0 %") || result.screens[3 + count()].contains("0%")
    );
    assert!(result.screens[3 + count()].contains("Потренировать ошибки"));
    assert!(result.screens[5 + count()].contains("wpm"));
}

#[test]
fn correct_exam_finishes_and_can_repeat() {
    let data = TestDataDir::new();
    let mut keys = vec![DOWN, DOWN, RIGHT, ENTER];
    let mut responses = Vec::new();
    for _ in 0..count() {
        responses.push((keys.len(), answer as fn(&str) -> Vec<u8>));
        keys.extend([b"".as_slice(), ENTER]);
    }
    let finish = keys.len() - 1;
    keys.extend([ENTER, ESC, ESC]);
    let result = run_course_capture(&keys, data.as_ref(), (80, 24), &responses);
    assert!(
        result.screens[finish].contains("100"),
        "{}",
        result.screens[finish]
    );
    assert!(!result.screens[finish].contains("Посмотреть ошибки"));
    assert!(!result.screens[finish].contains("Потренировать ошибки"));
    assert!(result.screens[finish + 1].contains("1 из"));
}

#[test]
fn malformed_or_unknown_choices_recover_and_narrow_terminal_works() {
    let data = TestDataDir::new();
    std::fs::write(data.as_ref().join("course-settings.json"), "broken").unwrap();
    let result = run(&[b"", ESC], &data);
    assert!(result.screens[0].contains("поврежд"));
    std::fs::write(
        data.as_ref().join("course-settings.json"),
        r#"{"module":"missing","topics":{},"mode":"Training"}"#,
    )
    .unwrap();
    let result = run_course_capture(&[b"", ENTER, ESC, ESC], data.as_ref(), (40, 24), &[]);
    assert!(result.screens[0].contains("Испанский"));
    let small = run_course_capture(&[b"", ESC], data.as_ref(), (40, 8), &[]);
    assert!(small.screens[0].contains("терминал") || small.screens[0].contains("окно"));
}

#[test]
fn each_module_is_available_and_round_trip_restores_topic() {
    let data = TestDataDir::new();
    let mut keys = vec![b"".as_slice()];
    keys.extend(std::iter::repeat_n(RIGHT, 7));
    keys.extend([DOWN, RIGHT, LEFT, ESC]);
    let result = run_course_capture(&keys, data.as_ref(), (180, 24), &[]);
    for (index, module) in catalog()["modules"].as_array().unwrap().iter().enumerate() {
        assert!(
            result.screens[index].contains(&format!(
                "< {}. {} >",
                index + 1,
                module["title"].as_str().unwrap()
            )),
            "{}",
            result.screens[index]
        );
    }
}

#[test]
fn long_module_introduction_is_clipped_centered_and_restored_at_narrow_width() {
    let data = TestDataDir::new();
    let mut keys = vec![RIGHT; 7];
    keys.extend([DOWN, RIGHT, DOWN, RIGHT, ESC]);
    let result = run_course_capture(&keys, data.as_ref(), (40, 24), &[]);
    let selected = &result.screens[10];
    let expected = "< 8. Me resulta fácil conversar en es… >";
    assert_eq!(
        selected
            .lines()
            .find(|line| line.contains("< 8."))
            .unwrap()
            .trim(),
        expected,
        "{selected}"
    );
    for line in selected.lines().filter(|line| {
        line.contains('<') || ["Модуль:", "Раздел:", "Режим:"].contains(&line.trim())
    }) {
        assert_centered(line, 40);
    }
    let restart = run_course_capture(&[b"", ESC], data.as_ref(), (40, 24), &[]);
    assert_eq!(restart.screens[0], *selected);
    assert!(restart.screens[0].contains("< Экзамен >"));
}

#[test]
fn partial_exam_result_and_error_subset_are_consistent() {
    let data = TestDataDir::new();
    let mut keys = vec![DOWN, DOWN, RIGHT, ENTER, b"".as_slice(), ENTER];
    keys.extend(std::iter::repeat_n(ENTER, count() - 1));
    let finish = keys.len() - 1;
    keys.extend([DOWN, ENTER, ESC, ESC]);
    let result = run_course_capture(&keys, data.as_ref(), (80, 24), &[(4, answer)]);
    assert!(
        result.screens[finish].contains(&format!("Правильно 1 из {}", count())),
        "{}",
        result.screens[finish]
    );
    assert!(
        result.screens[finish + 2].contains("wpm"),
        "{}",
        result.screens[finish + 2]
    );
}

#[test]
fn failed_save_is_visible_and_does_not_replace_previous_file() {
    let data = TestDataDir::new();
    let file = data.as_ref().join("blocked");
    std::fs::write(&file, b"keep").unwrap();
    let result = run_course_capture(&[RIGHT, ESC], &file, (80, 24), &[]);
    assert!(
        result.screens[0].contains("сохран"),
        "{}",
        result.screens[0]
    );
    assert_eq!(std::fs::read(file).unwrap(), b"keep");
}

#[test]
fn abrupt_process_exit_preserves_changed_choices_without_a_lesson() {
    let data = TestDataDir::new();
    let before =
        super::common::run_course_interrupted(&[RIGHT, DOWN, RIGHT, DOWN, RIGHT], data.as_ref());
    let restart = run(&[b"", ESC], &data);
    for value in before
        .screens
        .last()
        .unwrap()
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('<'))
    {
        assert!(restart.screens[0].contains(value), "{value}");
    }
}

#[test]
fn stale_topic_id_recovers_to_first_topic_and_can_start() {
    let data = TestDataDir::new();
    let module = catalog()["modules"][1]["id"].as_str().unwrap().to_owned();
    std::fs::write(data.as_ref().join("course-settings.json"),
        serde_json::to_vec(&serde_json::json!({"module":module,"topics":{module.clone():"removed"},"mode":"Training"})).unwrap()).unwrap();
    let result = run(&[b"", ENTER, ESC, ESC], &data);
    let catalog = catalog();
    let first = &catalog["modules"][1]["topics"][0];
    assert!(
        result.screens[0].contains("< 2. Quiero aprender español. >"),
        "{}",
        result.screens[0]
    );
    assert!(
        result.screens[0].contains(first["title"].as_str().unwrap()),
        "{}",
        result.screens[0]
    );
    assert!(
        result.screens[1].contains(first["entries"][0]["target"].as_str().unwrap()),
        "{}",
        result.screens[1]
    );
}

#[test]
fn narrow_long_answer_and_paged_results_keep_final_correction_reachable() {
    let data = TestDataDir::new();
    let wrong = format!("{}TAILMARK", "x".repeat(240));
    let mut keys = vec![DOWN, DOWN, RIGHT, ENTER];
    keys.extend([wrong.as_bytes(), ENTER]);
    for _ in 1..count() {
        keys.extend([b"".as_slice(), ENTER]);
    }
    let final_question = keys.len() - 2;
    keys.push(ENTER);
    keys.extend(std::iter::repeat_n(b"\x1b[6~".as_slice(), 30));
    let final_page = keys.len() - 1;
    keys.extend([ESC, ESC, ESC]);
    let result =
        super::common::run_course_waiting(&keys, data.as_ref(), (40, 24), &[(4, "TAILMARK")]);
    assert!(
        result.screens[4].contains("TAILMARK"),
        "{}",
        result.screens[final_question]
    );
    let catalog = catalog();
    let entry = catalog["modules"][0]["topics"][0]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| {
            result.screens[final_question]
                .lines()
                .any(|line| line.trim() == entry["ru"].as_str().unwrap())
        })
        .unwrap();
    assert!(
        result.screens[final_page].contains(entry["target"].as_str().unwrap()),
        "{}",
        result.screens[final_page]
    );
    assert!(
        result.screens[final_page].contains("Верный ответ"),
        "{}",
        result.screens[final_page]
    );
}

#[test]
fn training_input_uses_translation_layout_accents_and_case_rules_at_narrow_width() {
    let data = TestDataDir::new();
    // ENG physical keys spell Russian "вода", RU physical keys spell Spanish "agua".
    let r = run_course_capture(
        &[
            ENTER,
            b"djlf ",
            "фпгф".as_bytes(),
            ENTER,
            "кофе cafe".as_bytes(),
            ESC,
            ESC,
        ],
        data.as_ref(),
        (40, 24),
        &[],
    );
    use termwiz::color::ColorAttribute;
    for index in [2, 4] {
        assert!(
            r.cells[index]
                .iter()
                .filter(|(_, a)| a.foreground() == ColorAttribute::PaletteIndex(1))
                .count()
                == 0,
            "{}",
            r.screens[index]
        );
    }
    let r = run(&[ENTER, "В".as_bytes(), ESC, ESC], &data);
    let attrs = &r.cells[1]
        .iter()
        .find(|(t, a)| t == "в" && a.strikethrough())
        .unwrap()
        .1;
    assert_eq!(attrs.foreground(), ColorAttribute::PaletteIndex(1));
}

fn exam_correct_except_two(screen: &str) -> Vec<u8> {
    if screen.lines().any(|l| ["вода", "кофе"].contains(&l.trim())) {
        b"wrong".to_vec()
    } else {
        answer(screen)
    }
}

#[test]
fn exam_two_errors_train_and_repeat_only_the_missed_pairs() {
    let data = TestDataDir::new();
    let mut keys = vec![DOWN, DOWN, RIGHT, ENTER];
    let mut responses = vec![];
    for _ in 0..count() {
        responses.push((keys.len(), exam_correct_except_two as fn(&str) -> Vec<u8>));
        keys.extend([b"".as_slice(), ENTER]);
    }
    let result_at = keys.len() - 1;
    fn missed_pair(screen: &str) -> Vec<u8> {
        if screen.find("вода agua").unwrap_or(usize::MAX)
            < screen.find("кофе café").unwrap_or(usize::MAX)
        {
            "вода agua".as_bytes().to_vec()
        } else {
            "кофе cafe".as_bytes().to_vec()
        }
    }
    fn other_pair(screen: &str) -> Vec<u8> {
        if screen.find("вода agua").unwrap_or(usize::MAX)
            < screen.find("кофе café").unwrap_or(usize::MAX)
        {
            "кофе cafe".as_bytes().to_vec()
        } else {
            "вода agua".as_bytes().to_vec()
        }
    }
    keys.extend([
        b"\x1b[B\r".as_slice(),
        "вода agua".as_bytes(),
        ENTER,
        "кофе cafe".as_bytes(),
    ]);
    responses.push((result_at + 2, missed_pair));
    responses.push((result_at + 4, other_pair));
    let finish = keys.len() - 1;
    keys.extend([
        ENTER,
        "вода agua".as_bytes(),
        ENTER,
        "кофе cafe".as_bytes(),
        DOWN,
        ENTER,
        ENTER,
        ESC,
        ESC,
    ]);
    responses.push((finish + 2, missed_pair));
    responses.push((finish + 4, other_pair));
    let r = super::common::run_course_observing(
        &keys,
        data.as_ref(),
        &responses,
        &[
            (result_at, |s, _| s.contains("Правильно 18 из 20")),
            (result_at + 1, |s, _| s.contains("wpm")),
            (finish, |s, _| s.contains("Точность печати")),
            (finish + 1, |s, _| s.contains("wpm")),
            (finish + 4, |s, _| s.contains("Точность печати")),
            (finish + 6, |s, _| s.contains("Модуль")),
            (finish + 7, |s, _| s.contains("1 из 20")),
        ],
    );
    assert!(
        r.screens[result_at].contains("Правильно 18 из 20"),
        "{}",
        r.screens[result_at]
    );
    assert!(
        r.screens[finish].contains("100.0%"),
        "{}",
        r.screens[finish]
    );
    assert!(
        r.screens[finish + 4].contains("100.0%"),
        "{}",
        r.screens[finish + 4]
    );
    assert!(
        r.screens[finish + 7].contains("1 из 20"),
        "{}",
        r.screens[finish + 7]
    );
}

#[test]
fn exam_remains_strict_for_missing_accent() {
    fn no_accent(screen: &str) -> Vec<u8> {
        if screen.lines().any(|l| l.trim() == "кофе") {
            b"cafe".to_vec()
        } else {
            answer(screen)
        }
    }
    let data = TestDataDir::new();
    let mut keys = vec![DOWN, DOWN, RIGHT, ENTER];
    let mut responses = vec![];
    for _ in 0..count() {
        responses.push((keys.len(), no_accent as fn(&str) -> Vec<u8>));
        keys.extend([b"".as_slice(), ENTER]);
    }
    let at = keys.len() - 1;
    keys.extend([ESC, ESC]);
    let r = run_course_capture(&keys, data.as_ref(), (80, 24), &responses);
    assert!(
        r.screens[at].contains("Правильно 19 из 20"),
        "{}",
        r.screens[at]
    );
}

#[test]
fn training_final_combining_mark_matches_joined_event_and_freezes_time() {
    let catalog = catalog();
    let module = &catalog["modules"][0];
    let topic = &module["topics"][7];
    let text = topic["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            format!(
                "{} {}",
                e["ru"].as_str().unwrap(),
                e["target"].as_str().unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    assert!(text.ends_with('ú'));
    let prefix = &text[..text.len() - 'ú'.len_utf8()];
    for split in [false, true] {
        let data = TestDataDir::new();
        std::fs::write(data.as_ref().join("course-settings.json"),
            serde_json::to_vec(&serde_json::json!({"module":module["id"],"topics":{module["id"].as_str().unwrap():topic["id"]},"mode":"Training"})).unwrap()).unwrap();
        let mut chunks = text_keys(prefix);
        chunks.push(if split {
            b"u".to_vec()
        } else {
            "ú".as_bytes().to_vec()
        });
        if split {
            chunks.push("́".as_bytes().to_vec());
        }
        let mut keys = vec![ENTER];
        keys.extend(chunks.iter().map(Vec::as_slice));
        let end = keys.len() - 1;
        keys.extend([b"", ESC, ESC]);
        let r = run(&keys, &data);
        assert!(r.screens[end].contains("100.0%"), "{}", r.screens[end]);
        let time = |i: usize| {
            r.screens[i]
                .lines()
                .find(|l| l.contains("Время"))
                .unwrap()
                .trim()
        };
        assert_eq!(time(end), time(end + 1));
        if split {
            assert_eq!(time(end - 1), time(end));
        }
    }
}

fn assert_centered(line: &str, width: usize) {
    let start = line.chars().take_while(|c| c.is_whitespace()).count();
    // Terminal cells, including CJK and combining input, have a display width.
    let content = line.trim();
    let display = termwiz::cell::unicode_column_width(content, None);
    assert!(
        (2 * start + display).abs_diff(width) <= 1,
        "width={width} start={start} display={display} {line:?}"
    );
}

#[test]
fn exam_input_and_question_are_centered_in_terminal_cells() {
    for width in [40, 80, 81, 120] {
        let data = TestDataDir::new();
        let r = run_course_capture(
            &[
                DOWN,
                DOWN,
                RIGHT,
                ENTER,
                "á界".as_bytes(),
                b"\x7f",
                ESC,
                ESC,
            ],
            data.as_ref(),
            (width, 24),
            &[],
        );
        for index in [3, 4, 5] {
            let screen = &r.screens[index];
            for label in ["По-русски", "Ваш ответ"] {
                assert_centered(screen.lines().find(|l| l.trim() == label).unwrap(), width);
            }
            let cue = screen
                .lines()
                .find(|line| {
                    catalog()["modules"][0]["topics"][0]["entries"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|entry| line.trim() == entry["ru"].as_str().unwrap())
                })
                .unwrap();
            assert_centered(cue, width);
            let input = screen.lines().find(|l| l.trim().ends_with('_')).unwrap();
            assert_centered(input, width);
            assert!(
                !screen.contains("Верный ответ") && !screen.contains("Правильно"),
                "{screen}"
            );
        }
    }
}

#[test]
fn exam_result_opens_errors_on_request_and_returns_to_same_menu() {
    let data = TestDataDir::new();
    let mut keys = vec![DOWN, DOWN, RIGHT, ENTER];
    keys.extend(std::iter::repeat_n(ENTER, count()));
    let result_at = keys.len() - 1;
    keys.extend([
        ENTER, b"\x1b[6~", ESC, ENTER, ESC, UP, DOWN, DOWN, ENTER, ESC, ESC,
    ]);
    let r = run(&keys, &data);
    let result = &r.screens[result_at];
    assert!(
        result.contains("Посмотреть ошибки") && result.contains("0 %"),
        "{result}"
    );
    assert!(
        !result.contains("Верный ответ") && !result.contains("(пустой ответ)"),
        "{result}"
    );
    let review = &r.screens[result_at + 1];
    assert!(
        review.contains("(пустой ответ)") && review.contains("Верный ответ"),
        "{review}"
    );
    assert_eq!(r.screens[result_at + 3], *result);
    assert_eq!(r.screens[result_at + 4], *review);
    assert!(
        r.screens[result_at + 9].contains("wpm"),
        "{}",
        r.screens[result_at + 9]
    );
}
