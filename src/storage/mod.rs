mod catalog;
mod settings;
use crate::domain::Settings;
use catalog::bundled_datasets;

use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub(crate) struct Configuration {
    pub(crate) settings: Settings,
    conn: Connection,
}

impl std::fmt::Debug for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Configuration").finish()
    }
}

impl Configuration {
    #[cfg(test)]
    pub(crate) fn in_memory() -> Result<Self> {
        let config = Self {
            settings: Settings::default(),
            conn: Connection::open_in_memory()?,
        };
        config.init_tables()?;
        config.seed_datasets(bundled_datasets())?;
        Ok(config)
    }

    pub(crate) fn new() -> Result<Self> {
        let data_dir = Self::data_dir();

        std::fs::create_dir_all(&data_dir).ok();

        let db_path = data_dir.join("wordset.db");
        let conn = Connection::open(&db_path)?;

        let settings = Self::load_settings(&data_dir);

        let config = Self { settings, conn };
        config.init_tables()?;
        config.seed_datasets(bundled_datasets())?;
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
}
