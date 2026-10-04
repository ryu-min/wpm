# Working in this repository

## OpenSpec workflow

- Track every repository change through an OpenSpec change before editing project files, including documentation, tests, and CI.
- Use exploration to clarify an idea and a proposal to record the intended change. These are planning steps. Start implementation only after the user explicitly asks to apply or implement the change.
- Keep user-visible behavior requirements in `openspec/specs/`. For changes with no behavior requirements, such as documentation or tooling, set `skip_specs: true` in the change's `.openspec.yaml` instead of adding an artificial spec.
- Record implementation and acceptance checks in the change's `tasks.md`. Validate the change and complete its tasks before treating the work as finished.

## E2E acceptance tests

- Aim to cover all user-visible functionality with E2E tests. For each feature or behavior change, define observable acceptance scenarios in the OpenSpec change and add or update E2E tests that exercise them through the `wpm` binary. Passing relevant E2E tests is the acceptance evidence for the task.
- Test what the user sees and can do in the terminal, including important failure or exit paths where relevant. Do not rely only on internal implementation assertions for user-visible behavior.
- Use the existing tests in `tests/e2e_*.rs` and helpers in `tests/common/mod.rs`. They run `wpm` in a pseudo-terminal via `script`; use `TestDataDir` and `WPM_DATA_DIR` to keep test data isolated from the user's files.
- Run `cargo test --locked --all-features --all-targets` for implementation changes. E2E tests are currently Unix-only (`#![cfg(unix)]`), so Windows runs do not provide E2E coverage.
- If a behavior cannot reasonably be tested through the terminal, explain why in the change's `tasks.md` and specify another acceptance check. Do not leave a user-visible behavior change without verification.
