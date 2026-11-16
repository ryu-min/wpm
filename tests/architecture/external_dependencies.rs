use archunit::{assert_passes, project_files_in};

#[test]
fn ui_uses_only_its_external_dependencies() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("src/ui/**")
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
fn app_uses_only_its_external_dependencies() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("src/app/**")
            .should()
            .depend_on_external_modules()
            .matching("std")
            .matching("core")
            .matching("alloc")
            .matching("ratatui")
            .matching("crossterm")
            .matching("color_eyre")
    );
}

#[test]
fn domain_uses_only_its_external_dependencies() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("src/domain/**")
            .should()
            .depend_on_external_modules()
            .matching("std")
            .matching("core")
            .matching("alloc")
            .matching("serde")
            .matching("unicode_normalization")
            .matching("unicode_segmentation")
    );
}

#[test]
fn storage_uses_only_its_external_dependencies() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("src/storage/**")
            .should()
            .depend_on_external_modules()
            .matching("std")
            .matching("core")
            .matching("alloc")
            .matching("rusqlite")
            .matching("directories")
            .matching("serde")
            .matching("serde_json")
    );
}

#[test]
fn audio_uses_only_its_external_dependencies() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("src/audio/**")
            .should()
            .depend_on_external_modules()
            .matching("std")
            .matching("core")
            .matching("alloc")
    );
}

#[test]
fn dataset_uses_only_its_external_dependencies() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("src/dataset/**")
            .should()
            .depend_on_external_modules()
            .matching("std")
            .matching("core")
            .matching("alloc")
            .matching("serde")
            .matching("serde_json")
            .matching("unicode_normalization")
    );
}

#[test]
fn build_uses_only_its_external_dependencies() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("build.rs")
            .should()
            .depend_on_external_modules()
            .matching("std")
            .matching("core")
            .matching("alloc")
            .matching("serde_json")
    );
}

#[test]
fn main_uses_only_its_external_dependencies() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("src/main.rs")
            .should()
            .depend_on_external_modules()
            .matching("std")
            .matching("core")
            .matching("alloc")
            .matching("ratatui")
            .matching("color_eyre")
    );
}
