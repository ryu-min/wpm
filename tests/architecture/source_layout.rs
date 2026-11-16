use archunit::{assert_passes, pattern, project_files_in};

#[test]
fn discovered_sources_belong_to_a_documented_module() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("src/**")
            .should()
            .adhere_to(
                |file| {
                    let path = file.path.replace('\\', "/");
                    path == "src/main.rs"
                        || [
                            "app", "ui", "domain", "storage", "audio", "dataset", "tests",
                        ]
                        .iter()
                        .any(|root| path.starts_with(&format!("src/{root}/")))
                },
                "обнаруженный исходник должен принадлежать документированному модулю"
            )
    );
}

#[test]
fn production_and_build_source_do_not_suppress_architecture_analysis() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path(pattern("**/*.rs").except("src/tests/**"))
            .should()
            .adhere_to(
                |file| !file.content.contains("archunit:"),
                "исходники приложения и сборки не должны содержать маркер подавления анализа"
            )
    );
}
