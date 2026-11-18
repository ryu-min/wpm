use crate::storage::Configuration;
#[test]
fn bundled_spanish_catalog_meets_six_level_structural_contract() {
    use unicode_normalization::UnicodeNormalization;
    let config = Configuration::in_memory().unwrap();
    let mut ids = std::collections::BTreeSet::new();
    let mut lemmas = std::collections::BTreeSet::new();
    for level in ["a1", "a2", "b1", "b2", "c1", "c2"] {
        let set = config.get_vocabulary(&format!("ru_es_{level}")).unwrap();
        assert!((900..=1100).contains(&set.entries.len()));
        assert_eq!(set.level, level);
        for entry in set.entries {
            assert!(ids.insert(entry.id.clone()));
            assert!(lemmas.insert(entry.es.clone()));
            assert_eq!(entry.id, format!("ru_es_{level}:{}", entry.es));
            assert_eq!(entry.es, entry.es.nfc().collect::<String>());
            assert_eq!(entry.es, entry.es.to_lowercase());
            assert!(entry.es.chars().all(char::is_alphabetic));
            assert!(!entry.ru.trim().is_empty());
        }
    }
    assert!((5400..=6600).contains(&lemmas.len()));
}

#[cfg(test)]
mod case_settings_tests {
    use crate::domain::Settings;

    #[test]
    fn missing_case_fields_default_individually_without_resetting_settings() {
        let original = Settings {
            quick_start_time: 90,
            translation_time: 120,
            translation_set: "ru_es_b2".into(),
            typing_layout_conversion: true,
            typing_ignore_case: true,
            translation_ignore_case: true,
            ..Settings::default()
        };
        for missing in [
            vec!["typing_ignore_case", "translation_ignore_case"],
            vec!["typing_ignore_case"],
            vec!["translation_ignore_case"],
        ] {
            let mut json = serde_json::to_value(&original).unwrap();
            for name in &missing {
                json.as_object_mut().unwrap().remove(*name);
            }
            let loaded: Settings = serde_json::from_value(json).unwrap();
            let mut expected = serde_json::to_value(&original).unwrap();
            for name in missing {
                expected[name] = false.into();
            }
            assert_eq!(serde_json::to_value(loaded).unwrap(), expected);
        }
        assert!(!Settings::default().typing_ignore_case);
        assert!(!Settings::default().translation_ignore_case);
    }
}
