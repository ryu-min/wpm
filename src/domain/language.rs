//! Shared language rules, independent of screens, persistence.
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Language {
    Exact,
    Layout,
    Spanish,
}

pub(crate) fn units(text: &str) -> Vec<String> {
    text.nfc()
        .collect::<String>()
        .graphemes(true)
        .map(str::to_owned)
        .collect()
}

pub(crate) fn pop_grapheme(text: &mut String) {
    if let Some((offset, _)) = text.grapheme_indices(true).next_back() {
        text.truncate(offset);
    }
}

pub(crate) fn equivalent(input: &str, target: &str, language: Language) -> bool {
    let input: String = input.nfc().collect();
    let target: String = target.nfc().collect();
    if input == target {
        return true;
    }
    if language == Language::Exact {
        return false;
    }
    let mut i = input.chars();
    let mut t = target.chars();
    match (i.next(), i.next(), t.next(), t.next()) {
        (Some(i), None, Some(t), None) => {
            if interpret_char_for_target_layout(i, t) == t {
                return true;
            }
            if language == Language::Spanish {
                let simplify = |t| match t {
                    'ñ' => 'n',
                    'á' => 'a',
                    'é' => 'e',
                    'í' => 'i',
                    'ó' => 'o',
                    'ú' | 'ü' => 'u',
                    'Ñ' => 'N',
                    'Á' => 'A',
                    'É' => 'E',
                    'Í' => 'I',
                    'Ó' => 'O',
                    'Ú' | 'Ü' => 'U',
                    _ => t,
                };
                let base = simplify(t);
                simplify(i) == base || interpret_char_for_target_layout(i, base) == base
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Letter case variants retain a single grapheme and never shift punctuation keys.
pub(crate) fn case_variants(text: &str) -> Vec<String> {
    let original: String = text.nfc().collect();
    let mut variants = vec![original.clone()];
    if original.chars().any(char::is_alphabetic) {
        for variant in [original.to_lowercase(), original.to_uppercase()] {
            let variant: String = variant.nfc().collect();
            if variant.graphemes(true).count() == 1 && !variants.contains(&variant) {
                variants.push(variant);
            }
        }
    }
    variants
}

pub(crate) fn accepts(input: &str, target: &str, language: Language, ignore_case: bool) -> bool {
    equivalent(input, target, language)
        || (ignore_case
            && case_variants(input).iter().any(|input| {
                case_variants(target)
                    .iter()
                    .any(|target| equivalent(input, target, language))
            }))
}

pub(crate) fn matches(input: &str, target: &str, language: Language, ignore_case: bool) -> bool {
    let input = units(input);
    let target = units(target);
    input.len() == target.len()
        && input
            .iter()
            .zip(target)
            .all(|(input, target)| accepts(input, &target, language, ignore_case))
}

pub(crate) fn interpret_char_for_target_layout(input: char, target: char) -> char {
    if input == target {
        return input;
    }

    for &(russian, english) in LAYOUT_PAIRS {
        let russian_upper = russian.to_uppercase().next().unwrap();
        let english_upper = shifted_english_key(english);
        if (input == english && target == russian)
            || (input == russian && target == english)
            || (input == english_upper && target == russian_upper)
            || (input == russian_upper && target == english_upper)
        {
            return target;
        }
    }

    input
}

const LAYOUT_PAIRS: &[(char, char)] = &[
    ('й', 'q'),
    ('ц', 'w'),
    ('у', 'e'),
    ('к', 'r'),
    ('е', 't'),
    ('н', 'y'),
    ('г', 'u'),
    ('ш', 'i'),
    ('щ', 'o'),
    ('з', 'p'),
    ('х', '['),
    ('ъ', ']'),
    ('ф', 'a'),
    ('ы', 's'),
    ('в', 'd'),
    ('а', 'f'),
    ('п', 'g'),
    ('р', 'h'),
    ('о', 'j'),
    ('л', 'k'),
    ('д', 'l'),
    ('ж', ';'),
    ('э', '\''),
    ('я', 'z'),
    ('ч', 'x'),
    ('с', 'c'),
    ('м', 'v'),
    ('и', 'b'),
    ('т', 'n'),
    ('ь', 'm'),
    ('б', ','),
    ('ю', '.'),
    ('ё', '`'),
];

fn shifted_english_key(key: char) -> char {
    match key {
        '[' => '{',
        ']' => '}',
        ';' => ':',
        '\'' => '"',
        ',' => '<',
        '.' => '>',
        '`' => '~',
        _ => key.to_ascii_uppercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keyboard_contract() {
        for &(ru, en) in LAYOUT_PAIRS {
            assert!(equivalent(
                &ru.to_string(),
                &en.to_string(),
                Language::Layout
            ));
            assert!(equivalent(
                &en.to_string(),
                &ru.to_string(),
                Language::Layout
            ));
            assert!(equivalent(
                &ru.to_uppercase().to_string(),
                &shifted_english_key(en).to_string(),
                Language::Layout
            ));
        }
        assert!(matches("рщдф", "hola", Language::Spanish, false));
        assert!(!matches("хола", "hola", Language::Spanish, false));
    }
    #[test]
    fn case_variants_cover_every_layout_key_in_both_directions() {
        use crate::domain::TypingSession;
        let session = TypingSession::new(String::new())
            .with_layout_conversion(true)
            .with_ignore_case(true);
        for &(ru, en) in LAYOUT_PAIRS {
            for (input, target) in [
                (ru.to_uppercase().to_string(), en.to_string()),
                (en.to_string(), ru.to_uppercase().to_string()),
                (shifted_english_key(en).to_string(), ru.to_string()),
                (ru.to_string(), shifted_english_key(en).to_string()),
            ] {
                assert!(session.accepts(0, &input, &target), "{input} -> {target}");
            }
        }
    }
    #[test]
    fn whole_spanish_answers_cover_case_unicode_and_length() {
        for (plain, accented) in [
            ("n", "ñ"),
            ("a", "á"),
            ("e", "é"),
            ("i", "í"),
            ("o", "ó"),
            ("u", "ú"),
            ("u", "ü"),
            ("ú", "ü"),
        ] {
            for (input, target) in [(plain, accented), (accented, plain)] {
                assert!(matches(
                    &input.to_uppercase(),
                    target,
                    Language::Spanish,
                    true
                ));
                assert!(matches(
                    input,
                    &target.to_uppercase(),
                    Language::Spanish,
                    true
                ));
            }
        }
        assert!(matches("CAFE\u{301}", "café", Language::Spanish, true));
        for input in ["", "caf", "cafee", "cafi"] {
            assert!(!matches(input, "café", Language::Spanish, true));
        }
    }

    #[test]
    fn case_policy_preserves_physical_punctuation_keys() {
        for (input, target) in [(".", "."), ("ю", "."), ("Ю", ">"), ("Ж", ";")] {
            assert!(matches(input, target, Language::Spanish, true));
        }
        for (input, target) in [(">", "."), (":", ";")] {
            assert!(!matches(input, target, Language::Spanish, true));
        }
        assert!(!matches("A", "a", Language::Spanish, false));
        assert!(!matches("Ж", ";", Language::Spanish, false));
        assert!(matches("Ю", ">", Language::Spanish, false));
    }

    #[test]
    fn spanish_contract() {
        for (target, eng, ru) in [
            ("ñ", "n", "т"),
            ("á", "a", "ф"),
            ("é", "e", "у"),
            ("í", "i", "ш"),
            ("ó", "o", "щ"),
            ("ú", "u", "г"),
            ("ü", "u", "г"),
        ] {
            for input in [target, eng, ru] {
                assert!(matches(input, target, Language::Spanish, false));
            }
            assert!(!matches(eng, target, Language::Layout, false));
            assert!(!matches("x", target, Language::Spanish, false));
            assert!(!matches(
                &eng.to_uppercase(),
                target,
                Language::Spanish,
                false
            ));
            assert!(matches(
                &eng.to_uppercase(),
                &target.to_uppercase(),
                Language::Spanish,
                false
            ));
        }
        assert!(matches("si", "sí", Language::Spanish, false));
        assert!(matches("sí", "si", Language::Spanish, false));
        assert!(matches("ú", "ü", Language::Spanish, false));
        assert!(matches("ano", "año", Language::Spanish, false));
        assert!(matches("a\u{301}", "á", Language::Exact, false));
        let mut text = "a\u{301}".to_string();
        pop_grapheme(&mut text);
        assert!(text.is_empty());
    }
}
