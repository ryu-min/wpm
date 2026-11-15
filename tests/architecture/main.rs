//! Exact executable architecture rules live here; OpenSpec provides the overview.
//! ArchUnitRust checks recognized syntax across cfg branches, including inline tests.
//! It does not expand generated code, prove exhaustive inventory or resolve Cargo aliases
//! to package identities. Review new modules, manifest aliases and shared-model APIs.
use archunit::{assert_passes, pattern, project_files_in};

#[test]
fn leaf_modules_do_not_depend_on_other_source_files() {
    for source in [
        "src/menu_widget.rs",
        "src/result_widget.rs",
        "src/audio.rs",
        "src/language_input.rs",
        "src/dataset.rs",
    ] {
        assert_passes!(
            project_files_in(env!("CARGO_MANIFEST_DIR"))
                .in_file(source)
                .should_not()
                .depend_on_files()
                .in_path("**/*.rs")
        );
    }
}

#[test]
fn widgets_depend_only_on_their_shared_models() {
    for (widget, model) in [
        ("src/settings_widget.rs", "src/configuration.rs"),
        ("src/mode_select_widget.rs", "src/configuration.rs"),
        ("src/typing_widget.rs", "src/language_input.rs"),
        ("src/dictation_widget.rs", "src/dictation.rs"),
    ] {
        // This is file-level: Settings ownership within configuration still needs review.
        assert_passes!(
            project_files_in(env!("CARGO_MANIFEST_DIR"))
                .in_file(widget)
                .should()
                .depend_on_files()
                .in_path(model)
        );
    }
}

#[test]
fn configuration_depends_only_on_dataset_and_vocabulary() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_file("src/configuration.rs")
            .should_not()
            .depend_on_files()
            .in_path(pattern("**/*.rs").except_all(["src/dataset.rs", "src/vocabulary.rs"]))
    );
}

#[test]
fn dictation_depends_only_on_language_and_vocabulary() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_file("src/dictation.rs")
            .should_not()
            .depend_on_files()
            .in_path(pattern("**/*.rs").except_all(["src/language_input.rs", "src/vocabulary.rs"]))
    );
}

#[test]
fn vocabulary_keeps_runtime_and_inline_test_dependencies_explicit() {
    // Runtime needs language_input; existing inline tests exercise dictation and typing.
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_file("src/vocabulary.rs")
            .should_not()
            .depend_on_files()
            .in_path(pattern("**/*.rs").except_all([
                "src/language_input.rs",
                "src/dictation.rs",
                "src/typing_widget.rs"
            ]))
    );
}

#[test]
fn application_does_not_depend_on_the_entry_point() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_file("src/app.rs")
            .should_not()
            .depend_on_files()
            .in_path("src/main.rs")
    );
}

#[test]
fn build_script_depends_only_on_the_shared_dataset_parser() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_file("build.rs")
            .should()
            .depend_on_files()
            .in_path("src/dataset.rs")
    );
}

#[test]
fn non_typing_widgets_use_only_terminal_ui_dependencies() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path(pattern("src/*_widget.rs").except("src/typing_widget.rs"))
            .should()
            .depend_on_external_modules()
            .matching("std")
            .matching("core")
            .matching("alloc")
            .matching("ratatui")
            .matching("crossterm")
    );
}

#[test]
fn typing_uses_terminal_ui_and_unicode_normalization() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_file("src/typing_widget.rs")
            .should()
            .depend_on_external_modules()
            .matching("std")
            .matching("core")
            .matching("alloc")
            .matching("ratatui")
            .matching("crossterm")
            .matching("unicode_normalization")
    );
}

#[test]
fn configuration_uses_persistence_and_inline_test_normalization() {
    // unicode_normalization is used by existing inline dataset acceptance tests.
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_file("src/configuration.rs")
            .should()
            .depend_on_external_modules()
            .matching("std")
            .matching("core")
            .matching("alloc")
            .matching("rusqlite")
            .matching("directories")
            .matching("serde")
            .matching("serde_json")
            .matching("unicode_normalization")
    );
}

#[test]
fn support_modules_keep_external_dependencies_small() {
    for (source, allowed) in [
        ("src/audio.rs", &[][..]),
        ("src/dictation.rs", &[][..]),
        (
            "src/language_input.rs",
            &["unicode_normalization", "unicode_segmentation"][..],
        ),
        (
            "src/dataset.rs",
            &["serde", "serde_json", "unicode_normalization"][..],
        ),
        ("src/vocabulary.rs", &["serde"][..]),
        ("build.rs", &["serde_json"][..]),
    ] {
        let mut rule = project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_file(source)
            .should()
            .depend_on_external_modules()
            .matching("std")
            .matching("core")
            .matching("alloc");
        for dependency in allowed {
            rule = rule.matching(*dependency);
        }
        assert_passes!(rule);
    }
}

#[test]
fn production_and_build_source_do_not_suppress_architecture_analysis() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("**/*.rs")
            .should()
            .adhere_to(
                |file| !file.content.contains("archunit:"),
                "production/build source must not contain the reserved analyzer suppression marker"
            )
    );
}
