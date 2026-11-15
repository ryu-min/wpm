//! Closed v1 dataset format, shared by the build script and runtime adapters.
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    TypingRu,
    TypingEn,
    TranslationRuEn,
    SpanishWords,
    SpanishPhrases,
}
impl Profile {
    pub fn typing(self) -> bool {
        matches!(self, Self::TypingRu | Self::TypingEn)
    }
    pub fn spanish(self) -> bool {
        matches!(self, Self::SpanishWords | Self::SpanishPhrases)
    }
    pub fn language(self) -> &'static str {
        if self == Self::TypingRu {
            "ru"
        } else if self.spanish() {
            "es"
        } else {
            "en"
        }
    }
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dataset {
    pub schema_version: u32,
    pub id: String,
    pub description: String,
    pub profile: Profile,
    #[serde(default, deserialize_with = "optional_level")]
    pub level: Option<String>,
    pub entries: Vec<Value>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pair {
    pub id: String,
    pub ru: String,
    pub target: String,
}
fn optional_level<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    String::deserialize(deserializer).map(Some)
}
fn text(s: &str) -> bool {
    !s.is_empty()
        && s.trim() == s
        && s.nfc().collect::<String>() == s
        && !s.chars().any(char::is_control)
}
impl Dataset {
    pub fn parse(source: &str, raw: &str) -> Result<Self, String> {
        let data: Self = serde_json::from_str(raw).map_err(|e| format!("{source}: {e}"))?;
        data.validate().map_err(|e| format!("{source}: {e}"))?;
        let filename = std::path::Path::new(source)
            .file_stem()
            .and_then(|x| x.to_str());
        if filename != Some(data.id.as_str()) {
            return Err(format!("{source}: id must match filename: {}", data.id));
        }
        Ok(data)
    }
    fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("unsupported schema_version".into());
        }
        if !self
            .id
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_lowercase)
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err("invalid dataset id".into());
        }
        if !text(&self.description) || self.level.as_ref().is_some_and(|l| !text(l)) {
            return Err(
                "description/level must be nonempty NFC text without outer whitespace or controls"
                    .into(),
            );
        }
        if self.entries.is_empty() {
            return Err("entries must not be empty".into());
        }
        let mut ids = BTreeSet::new();
        let mut targets = BTreeSet::new();
        for (index, value) in self.entries.iter().enumerate() {
            let check = || -> Result<(), String> {
                if self.profile.typing() {
                    let word = value.as_str().ok_or("expected a word string")?;
                    if !text(word) || word.chars().any(char::is_whitespace) {
                        return Err("invalid typing word".into());
                    }
                } else {
                    let pair: Pair =
                        serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
                    if !text(&pair.id)
                        || !pair.id.starts_with(&format!("{}:", self.id))
                        || pair.id.len() <= self.id.len() + 1
                    {
                        return Err("invalid entry id/prefix".into());
                    }
                    if !ids.insert(pair.id.clone()) {
                        return Err(format!("duplicate entry id {}", pair.id));
                    }
                    if !text(&pair.ru)
                        || !pair
                            .ru
                            .chars()
                            .any(|c| ('А'..='я').contains(&c) || c == 'ё' || c == 'Ё')
                        || !text(&pair.target)
                    {
                        return Err(
                            "ru/target must be nonempty NFC text (ru must contain Russian text)"
                                .into(),
                        );
                    }
                    if !targets.insert(pair.target.clone()) {
                        return Err(format!("duplicate target {}", pair.target));
                    }
                    if self.profile.spanish() {
                        let allowed = |c: char| {
                            c.is_ascii_alphabetic()
                                || "áéíóúüñÁÉÍÓÚÜÑ".contains(c)
                                || (c == ' ' && self.profile == Profile::SpanishPhrases)
                        };
                        if !pair.target.chars().all(allowed) || pair.target.contains("  ") {
                            return Err("invalid Spanish target letters/spaces".into());
                        }
                        if self.profile == Profile::SpanishWords
                            && pair.target != pair.target.to_lowercase()
                        {
                            return Err("spanish-words requires lowercase lemmas".into());
                        }
                    }
                }
                Ok(())
            };
            let mut check = check;
            check().map_err(|e| format!("entry {}: {e}", index + 1))?;
        }
        Ok(())
    }
    pub fn words(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect()
    }
    pub fn pairs(&self) -> Vec<Pair> {
        self.entries
            .iter()
            .map(|v| serde_json::from_value(v.clone()).unwrap())
            .collect()
    }
}
