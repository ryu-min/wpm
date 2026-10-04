# Tasks

## 1. Command name and data compatibility

- [x] 1.1 Review the existing uncommitted edits, set the Cargo package and executable name to `wpm`, update `Cargo.lock`, and verify `cargo check --locked` reports package `wpm`.
- [x] 1.2 Update test helpers and project references that identify the binary as `wmp`; verify `cargo test --locked` runs against `CARGO_BIN_EXE_wpm` and `rg -n 'wmp|WMP' Cargo.toml Cargo.lock src tests openspec/config.yaml` finds no stale references.
- [x] 1.3 Replace the custom data-directory override with `WPM_DATA_DIR` while preserving the default data location; verify an integration test with a temporary override directory creates its data there and the existing settings tests pass.

## 2. Cargo installation and user documentation

- [x] 2.1 Replace the template README with a concise app description, Rust/Cargo prerequisite, Git and local-checkout installation commands, PATH note, `wpm` launch command, and license link; verify the commands and repository URL match the manifest and Git origin.
- [x] 2.2 Install from the local checkout to an isolated Cargo root with `cargo install --path . --locked`, verify its `bin/wpm` exists and is executable, and verify the application starts through the existing terminal integration tests.
- [x] 2.3 Extend GitHub Actions CI to run on every pull request and branch push and to install the checkout into an isolated Cargo root on Linux, macOS, and Windows; verify the workflow contains an assertion for `bin/wpm` or `bin/wpm.exe` on each platform and passes workflow syntax validation.
- [x] 2.4 Add a default-branch push job that runs the README's public Git installation command into an isolated Cargo root and asserts the `wpm` executable exists; verify its branch condition and URL match the repository's default branch and README, and check workflow syntax before the first published run.
- [x] 2.5 Run `cargo fmt -- --check`, `cargo test --locked`, and `openspec validate install-wpm-with-cargo`; verify all checks pass before marking the change complete.
