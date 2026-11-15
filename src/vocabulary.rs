use crate::language_input::{self, Language};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VocabularyEntry {
    pub id: String,
    pub ru: String,
    pub es: String,
}

#[derive(Debug, Clone)]
pub struct VocabularySet {
    pub id: String,
    pub source_language: String,
    pub target_language: String,
    pub level: String,
    pub entries: Vec<VocabularyEntry>,
}

impl VocabularySet {
    pub fn translation(&self) -> (String, Vec<Language>) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dictation::{Dictation, Phase},
        typing_widget::TypingWidget,
    };
    #[test]
    fn multiword_source_keeps_explicit_language_boundaries_and_shared_acceptance() {
        for (target, input) in [
            ("año", "ano"),
            ("año", "фтщ"),
            ("sí", "si"),
            ("si", "sí"),
            ("ü", "ú"),
            ("hola", "хола"),
        ] {
            let entry = VocabularyEntry {
                id: "stable".into(),
                ru: "новый год".into(),
                es: target.into(),
            };
            let set = VocabularySet {
                id: "set".into(),
                source_language: "ru".into(),
                target_language: "es".into(),
                level: "a1".into(),
                entries: vec![entry.clone()],
            };
            let (text, policy) = set.translation();
            let mut typing = TypingWidget::new(text).with_languages(policy);
            for ch in format!("новый год {input}").chars() {
                typing.add_char(ch);
            }
            let mut dictation = Dictation::new(vec![entry], 20).unwrap();
            dictation.phase = Phase::Input;
            dictation.heard = true;
            dictation.input = input.into();
            assert!(dictation.submit());
            assert_eq!(typing.get_accuracy() == 1.0, dictation.accepted);
        }
    }
}
