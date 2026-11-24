//! Resolve the offline course manifest to one compact runtime catalog.
use crate::dataset::{Dataset, Profile};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

#[derive(Deserialize, Serialize)]
struct Manifest {
    schema_version: u32,
    modules: Vec<Module>,
}
#[derive(Deserialize, Serialize)]
struct Module {
    id: String,
    title: String,
    topics: Vec<Topic>,
}
#[derive(Deserialize, Serialize)]
struct Topic {
    id: String,
    title: String,
    start: usize,
    end: usize,
    #[serde(skip_serializing)]
    dataset_id: Option<String>,
    #[serde(default)]
    unavailable: bool,
    #[serde(default, skip_serializing)]
    alternatives: BTreeMap<String, Vec<String>>,
    #[serde(skip_deserializing)]
    entries: Vec<Entry>,
}
#[derive(Serialize)]
struct Entry {
    id: String,
    ru: String,
    target: String,
    alternatives: Vec<String>,
}

pub fn assemble(dir: &Path) -> Result<Vec<u8>, String> {
    let file = dir.join("manifest.json");
    let read = |p: &Path| fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()));
    let mut catalog: Manifest =
        serde_json::from_str(&read(&file)?).map_err(|e| format!("{}: {e}", file.display()))?;
    if catalog.schema_version != 1 {
        return Err(format!(
            "unsupported course schema {}",
            catalog.schema_version
        ));
    }
    let datasets = dir.join("datasets");
    let mut unused = BTreeSet::new();
    for item in fs::read_dir(&datasets).map_err(|e| e.to_string())? {
        let path = item.map_err(|e| e.to_string())?.path();
        if path.extension().is_some_and(|s| s == "json") {
            unused.insert(path);
        }
    }
    let mut ids = BTreeSet::new();
    let mut words = BTreeSet::new();
    for module in &mut catalog.modules {
        if module.title.trim().is_empty()
            || module.id.is_empty()
            || !ids.insert(module.id.clone())
            || module.topics.is_empty()
        {
            return Err(format!("invalid module {}", module.id));
        }
        let mut next = 1;
        for topic in &mut module.topics {
            if topic.id.is_empty()
                || !ids.insert(topic.id.clone())
                || topic.title.trim().is_empty()
                || topic.start != next
                || topic.end < next
            {
                return Err(format!("invalid topic {} or section range", topic.id));
            }
            next = topic.end.checked_add(1).ok_or("invalid section range")?;
            if topic.unavailable && topic.dataset_id.is_some() {
                return Err(format!(
                    "unavailable topic {} refers to a dataset",
                    topic.id
                ));
            }
            if let Some(id) = &topic.dataset_id {
                let path = datasets.join(format!("{id}.json"));
                if !unused.remove(&path) {
                    return Err(format!("{}: missing or repeated dataset {id}", topic.id));
                }
                let dataset = Dataset::parse(path.to_str().ok_or("invalid path")?, &read(&path)?)?;
                if dataset.profile != Profile::SpanishPhrases {
                    return Err(format!("unexpected course dataset profile: {id}"));
                }
                for pair in dataset.pairs() {
                    if !words.insert(pair.target.to_lowercase()) {
                        return Err(format!("duplicate Spanish course target {}", pair.target));
                    }
                    let alternatives = topic.alternatives.remove(&pair.target).unwrap_or_default();
                    if alternatives.iter().any(|answer| {
                        answer.trim().is_empty()
                            || answer.trim() != answer
                            || answer.chars().any(char::is_control)
                    }) {
                        return Err(format!("invalid alternative answer for {}", pair.id));
                    }
                    topic.entries.push(Entry {
                        id: pair.id,
                        target: pair.target,
                        ru: pair.ru,
                        alternatives,
                    });
                }
            }
            if !topic.alternatives.is_empty() {
                return Err(format!("unknown alternative answer owner in {}", topic.id));
            }
        }
    }
    if catalog.modules.is_empty() || !unused.is_empty() {
        return Err(format!(
            "empty course or unreferenced course datasets: {unused:?}"
        ));
    }
    serde_json::to_vec(&catalog).map_err(|e| e.to_string())
}
