#[cfg(test)]
mod tests {
    use crate::domain::{Dictation, TypingSession};
    use crate::domain::{VocabularyEntry, VocabularySet};
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
            let mut typing = TypingSession::new(text).with_languages(policy);
            for ch in format!("новый год {input}").chars() {
                typing.add_char(ch);
            }
            let mut dictation = Dictation::new(vec![entry], 20).unwrap();
            dictation.audio_event(0, Ok(()));
            dictation.audio_event(0, Ok(()));
            for ch in input.chars() {
                dictation.push(ch);
            }
            assert!(dictation.submit());
            assert_eq!(typing.get_accuracy() == 1.0, dictation.accepted());
        }
    }
}
