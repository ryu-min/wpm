use super::{Language, language as language_input};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct VocabularyEntry {
    pub(crate) id: String,
    pub(crate) ru: String,
    pub(crate) es: String,
}

#[derive(Debug, Clone)]
pub(crate) struct VocabularySet {
    pub(crate) id: String,
    pub(crate) source_language: String,
    pub(crate) target_language: String,
    pub(crate) level: String,
    pub(crate) entries: Vec<VocabularyEntry>,
}

impl VocabularySet {
    pub(crate) fn translation(&self) -> (String, Vec<Language>) {
        debug_assert!(!self.id.is_empty());
        debug_assert_eq!(self.source_language, "ru");
        debug_assert!(!self.level.is_empty());
        let mut text = String::new();
        let mut policies = Vec::new();
        for entry in &self.entries {
            if !text.is_empty() {
                text.push('\n');
                policies.push(Language::Exact);
            }
            text.push_str(&entry.ru);
            policies.extend(vec![
                Language::Layout;
                language_input::units(&entry.ru).len()
            ]);
            text.push(' ');
            policies.push(Language::Exact);
            text.push_str(&entry.es);
            policies.extend(vec![
                if self.target_language == "es" {
                    Language::Spanish
                } else {
                    Language::Layout
                };
                language_input::units(&entry.es).len()
            ]);
        }
        (text, policies)
    }
}
