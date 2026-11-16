use archunit::{assert_passes, pattern, project_files_in};

#[test]
fn main_respects_module_boundaries() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("src/main.rs")
            .should_not()
            .depend_on_files()
            .in_path(pattern("**/*.rs").except_all([
                "src/main.rs",
                "src/app/**",
                "src/ui/**",
                "src/domain/**",
                "src/storage/**",
                "src/dataset/**",
                "src/tests/**"
            ]))
    );
}

#[test]
fn app_respects_module_boundaries() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("src/app/**")
            .should_not()
            .depend_on_files()
            .in_path(pattern("**/*.rs").except_all([
                "src/app/**",
                "src/ui/**",
                "src/domain/**",
                "src/storage/**"
            ]))
    );
}

#[test]
fn ui_respects_module_boundaries() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("src/ui/**")
            .should_not()
            .depend_on_files()
            .in_path(pattern("**/*.rs").except_all(["src/ui/**", "src/domain/**"]))
    );
}

#[test]
fn domain_respects_module_boundaries() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("src/domain/**")
            .should_not()
            .depend_on_files()
            .in_path(pattern("**/*.rs").except_all(["src/domain/**"]))
    );
}

#[test]
fn storage_respects_module_boundaries() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("src/storage/**")
            .should_not()
            .depend_on_files()
            .in_path(pattern("**/*.rs").except_all([
                "src/storage/**",
                "src/domain/**",
                "src/dataset/**"
            ]))
    );
}

#[test]
fn dataset_respects_module_boundaries() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("src/dataset/**")
            .should_not()
            .depend_on_files()
            .in_path(pattern("**/*.rs").except_all(["src/dataset/**"]))
    );
}

#[test]
fn build_respects_module_boundaries() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_path("build.rs")
            .should_not()
            .depend_on_files()
            .in_path(pattern("**/*.rs").except_all(["src/dataset/**"]))
    );
}
