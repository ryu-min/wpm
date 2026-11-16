use super::Configuration;
use crate::domain::Settings;
use std::path::Path;

impl Configuration {
    pub(super) fn load_settings(data_dir: &Path) -> Settings {
        let settings_path = data_dir.join("settings.json");
        if settings_path.exists()
            && let Ok(content) = std::fs::read_to_string(&settings_path)
            && let Ok(settings) = serde_json::from_str(&content)
        {
            return settings;
        }
        Settings::default()
    }

    pub(crate) fn save_settings(&self) -> std::io::Result<()> {
        let data_dir = Self::data_dir();
        std::fs::create_dir_all(&data_dir)?;

        let settings_path = data_dir.join("settings.json");
        let content = serde_json::to_string_pretty(&self.settings)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(settings_path, content)?;
        Ok(())
    }
}
