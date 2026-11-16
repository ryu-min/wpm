use super::Configuration;
use crate::{
    dataset::{Dataset, Profile},
    domain::{VocabularyEntry, VocabularySet},
};
use rusqlite::Result;
use serde::{Deserialize, Serialize};

pub(super) fn bundled_datasets() -> &'static [Dataset] {
    static DATA: std::sync::OnceLock<Vec<Dataset>> = std::sync::OnceLock::new();
    DATA.get_or_init(|| {
        const RAW: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/datasets.rs"));
        RAW.iter()
            .map(|(source, raw)| Dataset::parse(source, raw).expect("validated at build time"))
            .collect()
    })
}
#[derive(Debug, Serialize, Deserialize)]
struct TranslationPair {
    ru: String,
    en: String,
}

impl Configuration {
    pub(super) fn init_tables(&self) -> Result<()> {
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

    pub(super) fn seed_datasets(&self, datasets: &[Dataset]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        for d in datasets {
            let exists:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM word_sets WHERE name=?1 UNION SELECT 1 FROM translation_sets WHERE name=?1 UNION SELECT 1 FROM vocabulary_sets WHERE name=?1)",[&d.id],|r|r.get(0))?;
            if exists {
                continue;
            }
            let level = d.level.as_deref().unwrap_or("custom");
            if d.profile.typing() {
                tx.execute(
                    "INSERT INTO word_sets(name,language,word_count,words) VALUES(?1,?2,?3,?4)",
                    (
                        &d.id,
                        d.profile.language(),
                        d.entries.len(),
                        d.words().join(" "),
                    ),
                )?;
            } else if d.profile.spanish() {
                tx.execute(
                    "INSERT INTO vocabulary_sets VALUES(?1,'ru','es',?2)",
                    (&d.id, level),
                )?;
                for p in d.pairs() {
                    tx.execute(
                        "INSERT INTO vocabulary_entries VALUES(?1,?2,?3,?4)",
                        (&p.id, &d.id, &p.ru, &p.target),
                    )?;
                }
            } else {
                let pairs: Vec<_> = d
                    .pairs()
                    .into_iter()
                    .map(|p| TranslationPair {
                        ru: p.ru,
                        en: p.target,
                    })
                    .collect();
                tx.execute("INSERT INTO translation_sets(name,source_lang,target_lang,pair_count,pairs_json) VALUES(?1,'ru','en',?2,?3)",(&d.id,pairs.len(),serde_json::to_string(&pairs).map_err(invalid_data)?))?;
            }
        }
        tx.commit()
    }

    pub(super) fn profile(&self, name: &str) -> Result<Profile> {
        let mut query=self.conn.prepare("SELECT 'typing',language,'' FROM word_sets WHERE name=?1 UNION ALL SELECT 'translation',source_lang,target_lang FROM translation_sets WHERE name=?1 UNION ALL SELECT 'vocabulary',source_lang,target_lang FROM vocabulary_sets WHERE name=?1")?;
        let rows: Vec<(String, String, String)> = query
            .query_map([name], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect::<Result<_>>()?;
        if rows.len() != 1 {
            return Err(invalid_data("Missing or conflicting dataset storage"));
        }
        let (kind, source, target) = &rows[0];
        let actual = match (kind.as_str(), source.as_str(), target.as_str()) {
            ("typing", "ru", "") => Profile::TypingRu,
            ("typing", "en", "") => Profile::TypingEn,
            ("translation", "ru", "en") => Profile::TranslationRuEn,
            ("vocabulary", "ru", "es") => Profile::SpanishWords,
            _ => return Err(invalid_data("Unsupported dataset languages/type")),
        };
        if let Some(d) = bundled_datasets().iter().find(|d| d.id == name) {
            if d.profile != actual
                && !(d.profile == Profile::SpanishPhrases && actual == Profile::SpanishWords)
            {
                return Err(invalid_data(
                    "Dataset storage does not match declared profile",
                ));
            }
            return Ok(d.profile);
        }
        Ok(actual)
    }

    pub(crate) fn get_vocabulary(&self, name: &str) -> Result<VocabularySet> {
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

    pub(crate) fn get_shuffled_vocabulary(&self, name: &str) -> Result<VocabularySet> {
        let profile = self.profile(name)?;
        if profile.typing() {
            return Err(invalid_data("Typing dataset cannot start Translation"));
        }
        let mut set = if profile.spanish() {
            self.get_vocabulary(name)?
        } else {
            let pairs = self.get_translation_pairs(name)?;
            VocabularySet {
                id: name.into(),
                source_language: "ru".into(),
                target_language: "en".into(),
                level: bundled_datasets()
                    .iter()
                    .find(|d| d.id == name)
                    .map(|d| d.level.as_deref().unwrap_or("custom"))
                    .unwrap_or_else(|| name.rsplit('_').next().unwrap_or("custom"))
                    .into(),
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

    pub(crate) fn get_wordset_names(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM word_sets ORDER BY language, word_count")?;
        let names = stmt.query_map([], |row| row.get(0))?;
        let names: Vec<String> = names.collect::<Result<_>>()?;
        Ok(names
            .into_iter()
            .filter(|n| self.profile(n).is_ok_and(Profile::typing))
            .collect())
    }

    pub(crate) fn get_words(&self, wordset_name: &str) -> Result<Vec<String>> {
        if !self.profile(wordset_name)?.typing() {
            return Err(invalid_data("Expected Typing dataset"));
        }
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

    pub(crate) fn get_shuffled_words(&self, wordset_name: &str) -> Result<Vec<String>> {
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

    pub(crate) fn get_translation_set_names(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM translation_sets UNION SELECT name FROM vocabulary_sets ORDER BY name")?;
        let names = stmt.query_map([], |row| row.get(0))?;
        let names: Vec<String> = names.collect::<Result<_>>()?;
        Ok(names
            .into_iter()
            .filter(|n| self.profile(n).is_ok_and(|p| !p.typing()))
            .collect())
    }

    pub(crate) fn get_translation_pairs(&self, set_name: &str) -> Result<Vec<(String, String)>> {
        if self.profile(set_name)?.spanish() {
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
    use crate::domain::Settings;
    use rusqlite::Connection;
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
        config.seed_datasets(bundled_datasets()).unwrap();
        config.seed_datasets(bundled_datasets()).unwrap();
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
        assert_eq!(
            count,
            bundled_datasets()
                .iter()
                .filter(|d| d.profile.spanish())
                .count() as i64
        );
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
    fn dataset_seed_failure_rolls_back_every_new_set() {
        let config = memory();
        config.conn.execute_batch("CREATE TRIGGER reject_spanish BEFORE INSERT ON vocabulary_entries BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
        assert!(config.seed_datasets(bundled_datasets()).is_err());
        let count: i64 = config
            .conn
            .query_row("SELECT (SELECT count(*) FROM word_sets)+(SELECT count(*) FROM translation_sets)+(SELECT count(*) FROM vocabulary_sets)+(SELECT count(*) FROM vocabulary_entries)", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
    #[test]
    fn declarative_profiles_add_to_nonempty_storage_and_preserve_incompatible_rows() {
        let config = memory();
        config
            .conn
            .execute(
                "INSERT INTO word_sets VALUES(1,'personal','en',1,'mine')",
                [],
            )
            .unwrap();
        let mut datasets = Vec::new();
        for (i, p) in [
            "typing-ru",
            "typing-en",
            "translation-ru-en",
            "spanish-words",
            "spanish-phrases",
        ]
        .iter()
        .enumerate()
        {
            let id = format!("practice_{i}");
            let entries = if p.starts_with("typing-") {
                serde_json::json!(["word"])
            } else {
                serde_json::json!([{"id":format!("{id}:one"),"ru":"привет","target":"hola"}])
            };
            datasets.push(Dataset::parse(&format!("{id}.json"),&serde_json::json!({"schema_version":1,"id":id,"description":"test","profile":p,"entries":entries}).to_string()).unwrap());
        }
        config.seed_datasets(&datasets).unwrap();
        config.seed_datasets(&datasets).unwrap();
        for d in &datasets {
            if d.profile.typing() {
                assert_eq!(config.get_words(&d.id).unwrap(), vec!["word"]);
            } else {
                assert_eq!(
                    config.get_translation_pairs(&d.id).unwrap(),
                    vec![("привет".into(), "hola".into())]
                );
            }
        }
        assert_eq!(config.get_words("personal").unwrap(), vec!["mine"]);
        let id = &bundled_datasets()
            .iter()
            .find(|d| d.profile.spanish())
            .unwrap()
            .id;
        config.conn.execute("INSERT INTO word_sets(name,language,word_count,words) VALUES(?1,'en',1,'personal')",[id]).unwrap();
        config.seed_datasets(bundled_datasets()).unwrap();
        assert!(config.profile(id).is_err());
        let preserved: String = config
            .conn
            .query_row("SELECT words FROM word_sets WHERE name=?1", [id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(preserved, "personal");
    }
    #[test]
    fn old_settings_preserve_translation_and_typing_defaults() {
        let old = r#"{"quick_start_time":30,"quick_start_wordset":"ru_5000","translation_set":"ru_en_a2"}"#;
        let settings: Settings = serde_json::from_str(old).unwrap();
        assert_eq!(settings.translation_set, "ru_en_a2");
        assert_eq!(settings.quick_start_time, 30);
        assert_eq!(settings.quick_start_wordset, "ru_5000");
        assert_eq!(settings.quick_start_mode, "typing");
    }
}
