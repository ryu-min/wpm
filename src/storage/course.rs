use crate::domain::course::Choice;
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};
pub(crate) fn directory() -> PathBuf {
    std::env::var_os("WPM_ES_DATA_DIR")
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            directories::ProjectDirs::from("com", "wpm", "spanish-course")
                .map(|d| d.data_dir().into())
                .unwrap_or_else(|| PathBuf::from(".wpm-es"))
        })
}
pub(crate) fn load() -> (Choice, Option<String>) {
    let path = directory().join("course-settings.json");
    match fs::read(&path) {
        Ok(data) => match serde_json::from_slice(&data) {
            Ok(choice) => (choice, None),
            Err(_) => (
                Choice::default(),
                Some(
                    "Настройки повреждены. Применены начальные значения. Оригинал сохранён.".into(),
                ),
            ),
        },
        Err(e) if e.kind() == io::ErrorKind::NotFound => (Choice::default(), None),
        Err(e) => (
            Choice::default(),
            Some(format!("Не удалось прочитать настройки: {e}")),
        ),
    }
}
pub(crate) fn save(choice: &Choice) -> io::Result<()> {
    save_at(choice, &directory())
}
fn save_at(choice: &Choice, dir: &std::path::Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let path = dir.join("course-settings.json");
    if let Ok(data) = fs::read(&path)
        && serde_json::from_slice::<Choice>(&data).is_err()
    {
        let backup = dir.join(format!(
            "course-settings.corrupt-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::copy(&path, backup)?;
    }
    let temp = dir.join(format!("course-settings.{}.tmp", std::process::id()));
    let result = (|| {
        let mut file = fs::File::create(&temp)?;
        file.write_all(&serde_json::to_vec_pretty(choice)?)?;
        file.sync_all()?;
        fs::rename(&temp, &path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}

pub(crate) fn catalog() -> Result<crate::domain::course::Course, serde_json::Error> {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/spanish-course.json"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_atomic_write_preserves_previous_file() {
        let dir = std::env::temp_dir().join(format!(
            "wpm-course-store-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        save_at(&Choice::default(), &dir).unwrap();
        let old = fs::read(dir.join("course-settings.json")).unwrap();
        let temp = dir.join(format!("course-settings.{}.tmp", std::process::id()));
        fs::create_dir(&temp).unwrap();
        let choice = Choice {
            module: "m2".into(),
            ..Choice::default()
        };
        assert!(save_at(&choice, &dir).is_err());
        assert_eq!(fs::read(dir.join("course-settings.json")).unwrap(), old);
        fs::remove_dir(&temp).unwrap();
        save_at(&choice, &dir).unwrap();
        let saved: Choice =
            serde_json::from_slice(&fs::read(dir.join("course-settings.json")).unwrap()).unwrap();
        assert_eq!(saved.module, "m2");
        fs::remove_dir_all(dir).unwrap();
    }
}
