# Design

## Context

See `proposal.md` for motivation and `specs/local-installation/spec.md` for the user contract. The committed Cargo package is named `wmp`, which also determines the default binary name. Test helpers refer to that binary through Cargo's `CARGO_BIN_EXE_wmp` variable. Exercise data is embedded at compile time, and the application already creates its settings and SQLite database in a user data directory. CI runs Cargo tests on Linux, macOS, and Windows.

The working tree already contains uncommitted edits for this change from a premature implementation. The apply step must review them against these artifacts and verify them rather than assume they are complete.

## Goals / Non-Goals

**Goals:**

- Use Cargo's existing binary installation mechanism and expose one command, `wpm`.
- Keep the current default user data location so existing settings remain available.
- Keep the README short enough to serve as an installation entry point.
- Make installation failures visible on each pull request and branch push.

**Non-Goals:**

- Publish to crates.io, rename the GitHub repository, or build downloadable release binaries.
- Add a custom installer or automatic PATH modification.

## Decisions

### Rename the Cargo package

Change the package name to `wpm`, allowing Cargo's default binary target to use that name. Update the lockfile and test references to match. A separate `[[bin]]` target named `wpm` while keeping the package named `wmp` was considered, but would leave two conflicting project names in metadata and test tooling.

### Install directly from Git or a checkout

Document `cargo install --git https://github.com/ryu-min/wmp --locked` and `cargo install --path . --locked`. The Git URL stays as it is today; it identifies the repository, not the installed command. Cargo supplies the user-level installation location. A release archive or standalone install script would add distribution and maintenance work beyond this initial path.

### Exercise both installation sources in CI

Extend the existing GitHub Actions workflow to run on every pull request and branch push. In its Linux, macOS, and Windows matrix, install the checked-out source with `cargo install --path . --locked` into a temporary Cargo root and assert that the platform's `bin/wpm` or `bin/wpm.exe` exists. Use an isolated root so CI does not mistake a preinstalled command for this build's output. A separate job on default-branch pushes runs the README's Git installation command with a temporary root and checks the resulting executable. Running the public Git URL for pull requests was considered, but it would fetch the already published default branch rather than the proposed change; the checkout installation covers the actual PR revision.

### Rename the optional override without moving stored data

Use `WPM_DATA_DIR` for explicit data-directory selection and leave the existing `ProjectDirs` default unchanged. This makes the override consistent with the command name while avoiding a settings/database migration. Users who set `WMP_DATA_DIR` must change their environment configuration.

## Risks / Trade-offs

- A user without Rust cannot use this installation path -> State the Rust and Cargo prerequisite in the README.
- Cargo's binary directory may be missing from `PATH` -> State the PATH requirement and show the `wpm` launch command.
- The repository URL still contains `wmp` -> Explain its role through the installation command and keep the executable name consistently `wpm`.
- Renaming `WMP_DATA_DIR` can break existing custom setups -> Mark the change as breaking and document the replacement variable in the change.
- A local checkout test does not prove the remote Git install before these commits are pushed -> Verify local installation and test the Git command after publication.
- The Git check runs only after the default-branch push and therefore cannot gate that push in advance -> Use the checkout install as the PR gate and report any public Git failure in the default-branch CI run.

## Migration Plan

Update the manifest, lockfile, references, documentation, and CI workflow together. Validate the change, run the existing test suite, and install to an isolated Cargo root to confirm that the executable is `wpm`. Confirm the workflow runs for pull requests and pushes, with the public Git check limited to default-branch pushes. Existing default user data remains in place. Users with a custom `WMP_DATA_DIR` setting switch it to `WPM_DATA_DIR`.
