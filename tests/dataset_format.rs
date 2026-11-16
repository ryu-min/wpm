#[allow(dead_code)]
#[path = "../src/dataset/mod.rs"]
mod dataset;
use dataset::Dataset;
fn valid(profile: &str) -> serde_json::Value {
    serde_json::json!({"schema_version":1,"id":"practice","description":"Practice","profile":profile,"entries":if profile.starts_with("typing-"){serde_json::json!(["word"])}else{serde_json::json!([{"id":"practice:one","ru":"привет","target":"hola"}])}})
}
fn parse(v: &serde_json::Value) -> Result<Dataset, String> {
    Dataset::parse("practice.json", &v.to_string())
}
#[test]
fn all_profiles_validate_and_canonical_duplicates_remain_exact() {
    for p in [
        "typing-ru",
        "typing-en",
        "translation-ru-en",
        "spanish-words",
        "spanish-phrases",
    ] {
        assert!(parse(&valid(p)).is_ok());
    }
    let mut v = valid("spanish-phrases");
    v["entries"] = serde_json::json!([{"id":"practice:one","ru":"да","target":"sí"},{"id":"practice:two","ru":"если","target":"si"},{"id":"practice:three","ru":"привет","target":"Hola"},{"id":"practice:four","ru":"привет","target":"hola"}]);
    assert!(parse(&v).is_ok());
}
#[test]
fn invalid_metadata_and_entries_have_addressable_errors() {
    for (field, value) in [
        ("schema_version", serde_json::json!(2)),
        ("profile", serde_json::json!("other")),
        ("id", serde_json::json!("Bad-id")),
        ("description", serde_json::json!(" ")),
        ("level", serde_json::json!("")),
        ("level", serde_json::Value::Null),
        ("extra", serde_json::json!(true)),
        ("entries", serde_json::json!([])),
    ] {
        let mut v = valid("spanish-phrases");
        v[field] = value;
        let error = parse(&v).unwrap_err();
        assert!(error.contains("practice.json"), "{error}");
    }
    for (field, value) in [
        ("target", ""),
        ("target", " hola"),
        ("target", "hola  amigo"),
        ("target", "hola!"),
        ("target", "хола"),
        ("target", "a\u{301}"),
        ("ru", "hello"),
        ("ru", "да\nнет"),
        ("id", "other:x"),
        ("extra", "unknown"),
    ] {
        let mut v = valid("spanish-phrases");
        v["entries"][0][field] = serde_json::json!(value);
        let e = parse(&v).unwrap_err();
        assert!(e.contains("entry 1"), "{e}");
    }
    for duplicate in ["id", "target"] {
        let mut v = valid("spanish-phrases");
        let mut second = v["entries"][0].clone();
        second["id"] = serde_json::json!("practice:two");
        second["target"] = serde_json::json!("otro");
        second[duplicate] = v["entries"][0][duplicate].clone();
        v["entries"].as_array_mut().unwrap().push(second);
        assert!(parse(&v).unwrap_err().contains("entry 2"));
    }
    for target in ["Hola", "hola amigo"] {
        let mut v = valid("spanish-words");
        v["entries"][0]["target"] = serde_json::json!(target);
        assert!(parse(&v).is_err());
    }
    for word in ["", "two words", "word\n", "a\u{301}"] {
        let mut v = valid("typing-en");
        v["entries"][0] = serde_json::json!(word);
        assert!(parse(&v).unwrap_err().contains("entry 1"));
    }
}
