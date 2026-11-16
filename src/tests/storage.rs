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
