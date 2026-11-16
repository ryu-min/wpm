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

#[cfg(test)]
fn matches(input: &str, target: &str, language: Language) -> bool {
    let i = units(input);
    let t = units(target);
    i.len() == t.len() && i.iter().zip(t).all(|(a, b)| equivalent(a, &b, language))
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
        assert!(matches("рщдф", "hola", Language::Spanish));
        assert!(!matches("хола", "hola", Language::Spanish));
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
                assert!(matches(input, target, Language::Spanish));
            }
            assert!(!matches(eng, target, Language::Layout));
            assert!(!matches("x", target, Language::Spanish));
            assert!(!matches(&eng.to_uppercase(), target, Language::Spanish));
            assert!(matches(
                &eng.to_uppercase(),
                &target.to_uppercase(),
                Language::Spanish
            ));
        }
        assert!(matches("si", "sí", Language::Spanish));
        assert!(matches("sí", "si", Language::Spanish));
        assert!(matches("ú", "ü", Language::Spanish));
        assert!(matches("ano", "año", Language::Spanish));
        assert!(matches("a\u{301}", "á", Language::Exact));
        let mut text = "a\u{301}".to_string();
        pop_grapheme(&mut text);
        assert!(text.is_empty());
    }
}
