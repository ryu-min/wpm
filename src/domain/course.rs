use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Deserialize)]
pub(crate) struct Course {
    pub modules: Vec<Module>,
}
#[derive(Clone, Deserialize)]
pub(crate) struct Module {
    pub id: String,
    pub title: String,
    pub topics: Vec<Topic>,
}
#[derive(Clone, Deserialize)]
pub(crate) struct Topic {
    pub id: String,
    pub title: String,
    pub start: usize,
    pub end: usize,
    #[serde(default)]
    pub unavailable: bool,
    pub entries: Vec<Entry>,
}
#[derive(Clone, Deserialize)]
pub(crate) struct Entry {
    pub ru: String,
    pub target: String,
    #[serde(default)]
    pub alternatives: Vec<String>,
}
#[derive(Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub(crate) enum Mode {
    #[default]
    Training,
    Exam,
}
impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Training => "Тренировка",
            Self::Exam => "Экзамен",
        }
    }
}
impl Topic {
    pub fn empty_reason(&self) -> &'static str {
        if self.unavailable {
            "Словарь недоступен"
        } else {
            "Новых слов нет"
        }
    }
}
impl Entry {
    pub fn accepts(&self, answer: &str) -> bool {
        let answer = answer.trim();
        !answer.is_empty()
            && std::iter::once(&self.target)
                .chain(self.alternatives.iter())
                .any(|target| {
                    super::language::matches(
                        answer,
                        target.trim(),
                        super::language::Language::Spanish,
                        true,
                    )
                })
    }
}
pub(crate) fn backspace(s: &mut String) {
    if let Some((index, _)) = s.grapheme_indices(true).next_back() {
        s.truncate(index);
    }
}
pub(crate) fn shuffle<T>(items: &mut [T], mut seed: u64) {
    for i in (1..items.len()).rev() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        items.swap(i, (seed as usize) % (i + 1));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whole_answers_use_spanish_correspondences() {
        let e = Entry {
            ru: "кофе".into(),
            target: "café".into(),
            alternatives: vec![],
        };
        assert!(e.accepts(" CAFE\u{301} "));
        assert!(e.accepts("cafe"));
        assert!(e.accepts("СФАУ"));
        assert!(e.accepts("cфfУ"));
        for answer in ["", "caf", "cafee", "cafi"] {
            assert!(!e.accepts(answer), "{answer:?}");
        }
        let mut s = "cafe\u{301}".to_string();
        backspace(&mut s);
        assert_eq!(s, "caf");
    }
    #[test]
    fn alternatives_spaces_and_punctuation_use_full_comparison() {
        let mut entry = Entry {
            ru: "квартира".into(),
            target: "apartamento".into(),
            alternatives: vec![" piso ".into()],
        };
        assert!(entry.accepts(" ЗШЫЩ "));
        entry.alternatives.push(" ".into());
        assert!(!entry.accepts(" "));
        entry.target = "con leche".into();
        entry.alternatives.clear();
        for answer in ["con  leche", "conleche", "con leche!"] {
            assert!(!entry.accepts(answer), "{answer}");
        }
        for (target, answer) in [(".", "ю"), (">", "Ю"), (";", "Ж")] {
            entry.target = target.into();
            assert!(entry.accepts(answer));
        }
        for (target, answer) in [(".", ">"), (";", ":")] {
            entry.target = target.into();
            assert!(!entry.accepts(answer));
        }
        let mut session = Session::new(vec![entry], Mode::Exam, "Проверка".into());
        let answer = "  WrOnG  ".to_string();
        session.answers.push(answer.clone());
        assert_eq!(session.mistakes(), vec![0]);
        assert_eq!(session.answers, vec![answer]);
    }

    #[test]
    fn shuffle_is_a_permutation() {
        let mut a: Vec<_> = (0..20).collect();
        shuffle(&mut a, 121);
        assert_ne!(a, (0..20).collect::<Vec<_>>());
        a.sort();
        assert_eq!(a, (0..20).collect::<Vec<_>>());
    }
}

use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Choice {
    pub module: String,
    pub topics: BTreeMap<String, String>,
    pub mode: Mode,
}
pub(crate) struct Session {
    pub entries: Vec<Entry>,
    pub mode: Mode,
    pub index: usize,
    pub input: String,
    pub answers: Vec<String>,
    pub title: String,
}
impl Session {
    pub(crate) fn new(mut entries: Vec<Entry>, mode: Mode, title: String) -> Self {
        if mode == Mode::Exam {
            shuffle(
                &mut entries,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64
                    | 1,
            );
        }
        Self {
            entries,
            mode,
            index: 0,
            input: String::new(),
            answers: vec![],
            title,
        }
    }
    pub fn mistakes(&self) -> Vec<usize> {
        self.answers
            .iter()
            .enumerate()
            .filter_map(|(i, a)| (!self.entries[i].accepts(a)).then_some(i))
            .collect()
    }
}
pub(crate) struct Training {
    pub entries: Vec<Entry>,
    pub title: String,
    pub typing: super::TypingSession,
}
impl Training {
    pub fn new(entries: Vec<Entry>, title: String) -> Self {
        let (text, policies) = super::translation_text(
            entries.iter().map(|e| (e.ru.as_str(), e.target.as_str())),
            true,
        );
        Self {
            entries,
            title,
            typing: super::TypingSession::new(text).with_languages(policies),
        }
    }
}
#[cfg(test)]
mod training_tests {
    use super::*;
    #[test]
    fn translation_policy_covers_full_pairs() {
        let mut t = Training::new(
            vec![
                Entry {
                    ru: "год".into(),
                    target: "año".into(),
                    alternatives: vec![],
                },
                Entry {
                    ru: "кофе".into(),
                    target: "café".into(),
                    alternatives: vec![],
                },
            ],
            "Тест".into(),
        );
        assert_eq!(t.typing.target_text(), "год año\nкофе café");
        assert_eq!(t.typing.policy(0), super::super::Language::Layout);
        assert_eq!(t.typing.policy(3), super::super::Language::Exact);
        assert_eq!(t.typing.policy(5), super::super::Language::Spanish);
        assert_eq!(t.typing.policy(7), super::super::Language::Exact);
        for c in "год ano".chars() {
            t.typing.add_char(c);
        }
        assert!(!t.typing.is_complete());
        t.typing.insert_newline_if_expected();
        for c in "кофе cafe".chars() {
            t.typing.add_char(c);
        }
        assert!(t.typing.is_complete());
        assert_eq!(t.typing.get_accuracy(), 1.0);
    }
}
pub(crate) enum Screen {
    Home,
    Training(Training),
    TrainingResult(Training),
    Session(Session),
    Result(Session),
    Review(Session),
}
pub(crate) struct CourseApp {
    pub catalog: Course,
    pub choice: Choice,
    pub field: usize,
    pub screen: Screen,
    pub notice: Option<String>,
    pub save_failed: bool,
    pub action: usize,
    pub scroll: usize,
}

pub(crate) fn graphemes(text: &str) -> Vec<&str> {
    text.graphemes(true).collect()
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ResultAction {
    ViewErrors,
    TrainErrors,
    Repeat,
    Home,
}
impl ResultAction {
    pub fn label(self, mode: Mode) -> &'static str {
        match self {
            Self::ViewErrors => "Посмотреть ошибки",
            Self::TrainErrors => "Потренировать ошибки",
            Self::Repeat if mode == Mode::Exam => "Сдать экзамен ещё раз",
            Self::Repeat => "Повторить тренировку",
            Self::Home => "К выбору режима",
        }
    }
}
