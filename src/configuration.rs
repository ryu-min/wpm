use crate::vocabulary::{VocabularyEntry, VocabularySet};
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_quick_start_mode")]
    pub quick_start_mode: String,
    pub quick_start_time: u32,
    pub quick_start_wordset: String,
    #[serde(default = "default_quick_start_translation_set")]
    pub quick_start_translation_set: String,
    #[serde(default = "default_translation_time")]
    pub translation_time: u32,
    #[serde(default = "default_translation_set")]
    pub translation_set: String,
    #[serde(default)]
    pub typing_layout_conversion: bool,
    #[serde(default = "default_dictation_set")]
    pub dictation_set: String,
    #[serde(default = "default_dictation_length")]
    pub dictation_length: u32,
    #[serde(default = "default_dictation_set")]
    pub quick_start_dictation_set: String,
    #[serde(default = "default_dictation_length")]
    pub quick_start_dictation_length: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            quick_start_mode: default_quick_start_mode(),
            quick_start_time: 15,
            quick_start_wordset: "en_1000".to_string(),
            quick_start_translation_set: default_quick_start_translation_set(),
            translation_time: default_translation_time(),
            translation_set: default_translation_set(),
            typing_layout_conversion: false,
            dictation_set: default_dictation_set(),
            dictation_length: default_dictation_length(),
            quick_start_dictation_set: default_dictation_set(),
            quick_start_dictation_length: default_dictation_length(),
        }
    }
}

fn default_dictation_set() -> String {
    "ru_es_a1".into()
}
fn default_dictation_length() -> u32 {
    20
}

fn default_quick_start_mode() -> String {
    "typing".to_string()
}

fn default_quick_start_translation_set() -> String {
    "ru_en_a1".to_string()
}

fn default_translation_time() -> u32 {
    60
}

fn default_translation_set() -> String {
    "ru_en_a1".to_string()
}

pub struct Configuration {
    pub settings: Settings,
    conn: Connection,
}

#[derive(Debug, Serialize, Deserialize)]
struct TranslationPair {
    ru: String,
    en: String,
}

impl std::fmt::Debug for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Configuration").finish()
    }
}

impl Configuration {
    pub fn new() -> Result<Self> {
        let data_dir = Self::data_dir();

        std::fs::create_dir_all(&data_dir).ok();

        let db_path = data_dir.join("wordset.db");
        let conn = Connection::open(&db_path)?;

        let settings = Self::load_settings(&data_dir);

        let config = Self { settings, conn };
        config.init_tables()?;
        config.seed_data()?;
        Ok(config)
    }

    fn data_dir() -> PathBuf {
        if let Some(path) = std::env::var_os("WPM_DATA_DIR")
            && !path.is_empty()
        {
            return PathBuf::from(path);
        }

        directories::ProjectDirs::from("com", "wpm", "app")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap())
    }

    fn load_settings(data_dir: &Path) -> Settings {
        let settings_path = data_dir.join("settings.json");
        if settings_path.exists()
            && let Ok(content) = std::fs::read_to_string(&settings_path)
            && let Ok(settings) = serde_json::from_str(&content)
        {
            return settings;
        }
        Settings::default()
    }

    pub fn save_settings(&self) -> std::io::Result<()> {
        let data_dir = Self::data_dir();
        std::fs::create_dir_all(&data_dir)?;

        let settings_path = data_dir.join("settings.json");
        let content = serde_json::to_string_pretty(&self.settings)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(settings_path, content)?;
        Ok(())
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS word_sets (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                language TEXT NOT NULL,
                word_count INTEGER NOT NULL,
                words TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS translation_sets (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                source_lang TEXT NOT NULL,
                target_lang TEXT NOT NULL,
                pair_count INTEGER NOT NULL,
                pairs_json TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS vocabulary_sets (
                name TEXT PRIMARY KEY, source_lang TEXT NOT NULL, target_lang TEXT NOT NULL, level TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS vocabulary_entries (
                id TEXT PRIMARY KEY, set_name TEXT NOT NULL REFERENCES vocabulary_sets(name),
                ru TEXT NOT NULL, es TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    fn seed_data(&self) -> Result<()> {
        let wordset_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM word_sets", [], |row| row.get(0))?;

        if wordset_count == 0 {
            let word_files = vec![
                ("en_1000", "en", include_str!("../resources/en_1000.txt")),
                ("en_10000", "en", include_str!("../resources/en_10000.txt")),
                ("ru_5000", "ru", include_str!("../resources/ru_5000.txt")),
                ("ru_10000", "ru", include_str!("../resources/ru_10000.txt")),
                ("ru_50000", "ru", include_str!("../resources/ru_50000.txt")),
            ];

            let mut stmt = self.conn.prepare(
                "INSERT INTO word_sets (name, language, word_count, words) VALUES (?1, ?2, ?3, ?4)",
            )?;

            for (name, lang, content) in word_files {
                let words: Vec<&str> = content.lines().filter(|s| !s.is_empty()).collect();
                let word_count = words.len() as i64;
                let words_blob = words.join(" ");
                stmt.execute((name, lang, word_count, words_blob))?;
            }
        }

        let translation_sets: Vec<(&str, &str)> = vec![
            ("ru_en_a1", include_str!("../resources/ru_en_a1.json")),
            ("ru_en_a2", include_str!("../resources/ru_en_a2.json")),
            ("ru_en_b1", include_str!("../resources/ru_en_b1.json")),
            ("ru_en_b2", include_str!("../resources/ru_en_b2.json")),
            ("ru_en_c1", include_str!("../resources/ru_en_c1.json")),
            ("ru_en_c2", include_str!("../resources/ru_en_c2.json")),
            ("ru_en_basic", include_str!("../resources/ru_en_basic.json")),
        ];

        for (set_name, pairs_raw) in translation_sets {
            let existing: i64 = self.conn.query_row(
                "SELECT COUNT(*) FROM translation_sets WHERE name = ?1",
                [set_name],
                |row| row.get(0),
            )?;
            if existing > 0 {
                continue;
            }
            let pairs: Vec<TranslationPair> = serde_json::from_str(pairs_raw).unwrap_or_default();
            let pair_count = pairs.len() as i64;
            let pairs_json = serde_json::to_string(&pairs).unwrap_or_else(|_| "[]".to_string());
            self.conn.execute(
                "INSERT INTO translation_sets (name, source_lang, target_lang, pair_count, pairs_json) VALUES (?1, ?2, ?3, ?4, ?5)",
                (set_name, "ru", "en", pair_count, pairs_json),
            )?;
        }
        self.seed_spanish()?;
        Ok(())
    }

    fn seed_spanish(&self) -> Result<()> {
        let files = [
            ("ru_es_a1", "a1", include_str!("../resources/ru_es_a1.json")),
            ("ru_es_a2", "a2", include_str!("../resources/ru_es_a2.json")),
            ("ru_es_b1", "b1", include_str!("../resources/ru_es_b1.json")),
            ("ru_es_b2", "b2", include_str!("../resources/ru_es_b2.json")),
            ("ru_es_c1", "c1", include_str!("../resources/ru_es_c1.json")),
            ("ru_es_c2", "c2", include_str!("../resources/ru_es_c2.json")),
            (
                "ru_es_cafe_beginner",
                "beginner",
                include_str!("../resources/ru_es_cafe_beginner.json"),
            ),
        ];
        let tx = self.conn.unchecked_transaction()?;
        for (name, level, raw) in files {
            let exists: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM vocabulary_sets WHERE name=?1)",
                [name],
                |r| r.get(0),
            )?;
            if exists {
                continue;
            }
            let entries: Vec<VocabularyEntry> = serde_json::from_str(raw).map_err(invalid_data)?;
            tx.execute(
                "INSERT INTO vocabulary_sets VALUES (?1,'ru','es',?2)",
                (name, level),
            )?;
            for entry in entries {
                tx.execute(
                    "INSERT INTO vocabulary_entries VALUES (?1,?2,?3,?4)",
                    (&entry.id, name, &entry.ru, &entry.es),
                )?;
            }
        }
        tx.commit()
    }

    pub fn get_vocabulary(&self, name: &str) -> Result<VocabularySet> {
        let (source_language, target_language, level): (String, String, String) =
            self.conn.query_row(
                "SELECT source_lang,target_lang,level FROM vocabulary_sets WHERE name=?1",
                [name],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )?;
        let mut stmt = self
            .conn
            .prepare("SELECT id,ru,es FROM vocabulary_entries WHERE set_name=?1 ORDER BY id")?;
        let entries: Vec<VocabularyEntry> = stmt
            .query_map([name], |r| {
                Ok(VocabularyEntry {
                    id: r.get(0)?,
                    ru: r.get(1)?,
                    es: r.get(2)?,
                })
            })?
            .collect::<Result<_>>()?;
        if entries.is_empty()
            || entries
                .iter()
                .any(|e| e.id.is_empty() || e.ru.trim().is_empty() || e.es.trim().is_empty())
        {
            return Err(invalid_data("Empty or damaged vocabulary set"));
        }
        Ok(VocabularySet {
            id: name.into(),
            source_language,
            target_language,
            level,
            entries,
        })
    }

    pub fn get_shuffled_vocabulary(&self, name: &str) -> Result<VocabularySet> {
        let mut set = if name.starts_with("ru_es_") {
            self.get_vocabulary(name)?
        } else {
            let pairs = self.get_translation_pairs(name)?;
            VocabularySet {
                id: name.into(),
                source_language: "ru".into(),
                target_language: "en".into(),
                level: name.rsplit('_').next().unwrap_or("").into(),
                entries: pairs
                    .into_iter()
                    .enumerate()
                    .map(|(i, (ru, en))| VocabularyEntry {
                        id: format!("{name}:{i}"),
                        ru,
                        es: en,
                    })
                    .collect(),
            }
        };
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as usize;
        shuffle(&mut set.entries, &mut seed_rng(seed));
        Ok(set)
    }

    pub fn get_wordset_names(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM word_sets ORDER BY language, word_count")?;
        let names = stmt.query_map([], |row| row.get(0))?;
        names.collect()
    }

    pub fn get_words(&self, wordset_name: &str) -> Result<Vec<String>> {
        let words_blob: String = self.conn.query_row(
            "SELECT words FROM word_sets WHERE name = ?1",
            [wordset_name],
            |row| row.get(0),
        )?;
        Ok(words_blob
            .split_whitespace()
            .map(|s| s.to_string())
            .collect())
    }

    pub fn get_shuffled_words(&self, wordset_name: &str) -> Result<Vec<String>> {
        let mut words = self.get_words(wordset_name)?;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);
        let seed = hasher.finish() as usize;
        let mut rng = seed_rng(seed);
        shuffle(&mut words, &mut rng);
        Ok(words)
    }

    pub fn get_translation_set_names(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM translation_sets UNION SELECT name FROM vocabulary_sets ORDER BY name")?;
        let names = stmt.query_map([], |row| row.get(0))?;
        names.collect()
    }

    pub fn get_translation_pairs(&self, set_name: &str) -> Result<Vec<(String, String)>> {
        if set_name.starts_with("ru_es_") {
            return Ok(self
                .get_vocabulary(set_name)?
                .entries
                .into_iter()
                .map(|e| (e.ru, e.es))
                .collect());
        }
        let pairs_json: String = self.conn.query_row(
            "SELECT pairs_json FROM translation_sets WHERE name = ?1",
            [set_name],
            |row| row.get(0),
        )?;
        let pairs: Vec<TranslationPair> =
            serde_json::from_str(&pairs_json).map_err(invalid_data)?;
        if pairs.is_empty()
            || pairs
                .iter()
                .any(|p| p.ru.trim().is_empty() || p.en.trim().is_empty())
        {
            return Err(invalid_data("Empty or damaged translation set"));
        }
        Ok(pairs.into_iter().map(|p| (p.ru, p.en)).collect())
    }
}

fn seed_rng(seed: usize) -> Rand {
    Rand { state: seed }
}

struct Rand {
    state: usize,
}

impl Rand {
    fn next(&mut self) -> usize {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state >> 16
    }
}

fn shuffle<T>(v: &mut [T], rng: &mut Rand) {
    for i in (1..v.len()).rev() {
        let j = rng.next() % (i + 1);
        v.swap(i, j);
    }
}

fn invalid_data(error: impl std::fmt::Display) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            error.to_string(),
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn memory() -> Configuration {
        let config = Configuration {
            settings: Settings::default(),
            conn: Connection::open_in_memory().unwrap(),
        };
        config.init_tables().unwrap();
        config
    }
    #[test]
    fn additive_migration_preserves_old_json_and_seed_is_idempotent() {
        let config = memory();
        let old = "[{\"ru\":\"мой кот\",\"en\":\"cat\"}]";
        config
            .conn
            .execute(
                "INSERT INTO translation_sets VALUES (41,'ru_en_a2','ru','en',1,?1)",
                [old],
            )
            .unwrap();
        config.seed_data().unwrap();
        config.seed_data().unwrap();
        assert_eq!(
            config.get_translation_pairs("ru_en_a2").unwrap(),
            vec![("мой кот".into(), "cat".into())]
        );
        let stored: String = config
            .conn
            .query_row(
                "SELECT pairs_json FROM translation_sets WHERE id=41",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stored, old);
        let count: i64 = config
            .conn
            .query_row("SELECT count(*) FROM vocabulary_sets", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 7);
        let canonical = config.get_vocabulary("ru_es_a1").unwrap();
        let shuffled = config.get_shuffled_vocabulary("ru_es_a1").unwrap();
        let ids = |set: VocabularySet| {
            set.entries
                .into_iter()
                .map(|e| (e.id, e.es))
                .collect::<std::collections::BTreeMap<_, _>>()
        };
        assert_eq!(ids(canonical.clone()), ids(shuffled));
        assert_eq!(
            (
                canonical.id.as_str(),
                canonical.source_language.as_str(),
                canonical.level.as_str()
            ),
            ("ru_es_a1", "ru", "a1")
        );
    }
    #[test]
    fn bundled_spanish_catalog_meets_six_level_structural_contract() {
        use unicode_normalization::UnicodeNormalization;
        let config = memory();
        config.seed_spanish().unwrap();
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
    #[test]
    fn invalid_selected_sets_report_errors_without_repairing_user_data() {
        let config = memory();
        config
            .conn
            .execute(
                "INSERT INTO vocabulary_sets VALUES ('ru_es_a1','ru','es','a1')",
                [],
            )
            .unwrap();
        assert!(config.get_vocabulary("ru_es_a1").is_err());
        config
            .conn
            .execute(
                "INSERT INTO translation_sets VALUES (1,'broken','ru','en',1,'not json')",
                [],
            )
            .unwrap();
        assert!(config.get_translation_pairs("broken").is_err());
    }
    #[test]
    fn spanish_seed_failure_rolls_back_every_new_set() {
        let config = memory();
        config.conn.execute_batch("CREATE TRIGGER reject_spanish BEFORE INSERT ON vocabulary_entries BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
        assert!(config.seed_spanish().is_err());
        let count: i64 = config
            .conn
            .query_row("SELECT count(*) FROM vocabulary_sets", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
    #[test]
    fn old_settings_get_independent_dictation_defaults() {
        let old = r#"{"quick_start_time":30,"quick_start_wordset":"ru_5000","translation_set":"ru_en_a2"}"#;
        let settings: Settings = serde_json::from_str(old).unwrap();
        assert_eq!(settings.translation_set, "ru_en_a2");
        assert_eq!(settings.dictation_set, "ru_es_a1");
        assert_eq!(settings.dictation_length, 20);
        assert_eq!(settings.quick_start_dictation_length, 20);
    }
}
