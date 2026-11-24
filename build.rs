#[path = "src/dataset/course_build.rs"]
mod course_build;
#[allow(dead_code)]
#[path = "src/dataset/mod.rs"]
mod dataset;

fn main() {
    let course_dir = std::path::Path::new("resources/course");
    for input in ["manifest.json", "datasets"] {
        println!(
            "cargo:rerun-if-changed={}",
            course_dir.join(input).display()
        );
    }
    println!("cargo:rerun-if-changed=src/dataset/course_build.rs");
    let course =
        course_build::assemble(course_dir).unwrap_or_else(|e| panic!("Spanish course: {e}"));
    std::fs::write(
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("spanish-course.json"),
        course,
    )
    .unwrap();
    println!("cargo:rerun-if-changed=resources/datasets");
    println!("cargo:rerun-if-changed=src/dataset/mod.rs");
    let mut ids = std::collections::BTreeMap::<String, String>::new();
    let mut paths: Vec<_> = std::fs::read_dir("resources/datasets")
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    let files: Vec<_> = paths
        .into_iter()
        .map(|path| {
            let source = path.to_str().unwrap().to_owned();
            println!("cargo:rerun-if-changed={source}");
            let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{source}: {e}"));
            (source, raw)
        })
        .collect();
    // Diagnose both sources before filename validation, regardless of sorting:
    // one conflicting file necessarily has a filename different from its id.
    for (source, raw) in &files {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw)
            && let Some(id) = value["id"].as_str()
            && let Some(previous) = ids.insert(id.into(), source.clone())
        {
            panic!("{source}: duplicate dataset id {id} conflicts with {previous}");
        }
    }
    let mut output = String::from("&[\n");
    for (source, raw) in files {
        dataset::Dataset::parse(&source, &raw).unwrap_or_else(|e| panic!("{e}"));
        output.push_str(&format!("({source:?}, {raw:?}),\n"));
    }
    output.push(']');
    std::fs::write(
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("datasets.rs"),
        output,
    )
    .unwrap();
}
