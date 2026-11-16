use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Settings {
    #[serde(default = "default_quick_start_mode")]
    pub(crate) quick_start_mode: String,
    pub(crate) quick_start_time: u32,
    pub(crate) quick_start_wordset: String,
    #[serde(default = "default_quick_start_translation_set")]
    pub(crate) quick_start_translation_set: String,
    #[serde(default = "default_translation_time")]
    pub(crate) translation_time: u32,
    #[serde(default = "default_translation_set")]
    pub(crate) translation_set: String,
    #[serde(default)]
    pub(crate) typing_layout_conversion: bool,
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
        }
    }
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
