#[cfg(test)]
mod tests {
    use crate::domain::TypingSession;
    use crate::domain::{VocabularyEntry, VocabularySet};
    #[test]
    fn multiword_source_keeps_explicit_language_boundaries_and_translation_acceptance() {
        for (target, input, accepted) in [
            ("año", "ano", true),
            ("año", "фтщ", true),
            ("sí", "si", true),
            ("si", "sí", true),
            ("ü", "ú", true),
            ("hola", "хола", false),
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
            let mut typing = TypingSession::new(text).with_languages(policy);
            for ch in format!("новый год {input}").chars() {
                typing.add_char(ch);
            }
            assert_eq!(typing.get_accuracy() == 1.0, accepted);
        }
    }
}
