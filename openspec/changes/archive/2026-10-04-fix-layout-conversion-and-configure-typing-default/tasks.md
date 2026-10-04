# Tasks

## 1. Write terminal acceptance tests first

- [x] 1.1 Add E2E tests in `tests/e2e_typing.rs` or `tests/e2e_result.rs` using `TestDataDir` and `run_wpm_capture` for `;` against `ж`, plus a deterministic fixture covering all 33 Russian letters and their English-layout keys in both directions (including punctuation-position keys and `ё`); verify the new tests compile with `cargo test --locked --all-features --all-targets --no-run`.
- [x] 1.2 Run the new layout E2E tests against the unchanged application and record the failing `ж` scenario and any other failed key pairs in this task's acceptance notes before editing `src/`; verify that the failure is the terminal-observable result, not a test harness or fixture error.
- [x] 1.3 Add E2E scenarios for Translation launched through each menu path, Typing conversion off by default through Quick Start and Select Mode, direct-layout input, saved Typing option on, restart behavior, and an older `settings.json` without the field; verify the new tests compile and their current failures reflect the intended behavior.

## 2. Fix conversion against the failing tests

- [x] 2.1 Make `TypingWidget` conversion opt-in per exercise and correct target-aware mapping for all alphabet key pairs, including punctuation input/target and `ё`; compare character counts rather than UTF-8 byte lengths when completing mixed-layout exercises; verify the application and new E2E tests compile with `cargo test --locked --all-features --all-targets --no-run`.
- [x] 2.2 Add the backward-compatible Typing preference in `src/configuration.rs` and thread the conversion choice through every exercise creation and restart path in `src/app.rs`, always on for Translation and based on the preference otherwise; verify the `ж`, full-alphabet, menu-path, default-off, direct-input, and restart E2E scenarios pass.

## 3. Expose and persist the Typing preference

- [x] 3.1 Add a visible on/off control for the Typing preference in the Settings UI and save it with other settings; verify E2E covers save, relaunch, old settings files, enabled Typing, and Translation remaining enabled while the Typing option is off.
- [x] 3.2 Update any existing Settings E2E navigation sequences affected by the new control; verify `cargo test --locked --all-features --all-targets` passes, including all relevant E2E tests. Existing navigation sequences required no edits; the new section follows the existing sections.

## 4. Acceptance and validation

- [x] 4.1 Run `openspec validate fix-layout-conversion-and-configure-typing-default --strict` and the full `cargo test --locked --all-features --all-targets`; verify both pass and record the commands and result here. The terminal E2E tests validate characters received by `wpm`; a physical Mac keyboard smoke check is optional because OS keyboard layout selection is outside the terminal input that `wpm` controls.

## Acceptance notes

- Baseline before any `src/` edits: `cargo test --locked --all-features --test e2e_result layout_translation -- --nocapture` failed all three new E2E tests. The `;`/`ж` and full Russian-alphabet cases remained on the typing screen instead of reaching a correct result. The reverse full-alphabet case reached the result screen with 83% accuracy. The deterministic fixtures and terminal screen captures confirm these are observable application failures. Existing E2E harness tests compile. The mixed UTF-8/ASCII target also exposed byte-based completion in `TypingWidget::is_complete`, so task 2.1 explicitly covers character-count completion.
- Before `src/` edits, `cargo test --locked --all-features --test e2e_layout_policy -- --nocapture` compiled and failed all five new policy tests as expected: Translation cannot accept `;`, Typing still converts by default, and the Typing Settings section does not yet exist.
- Final acceptance: `cargo test --locked --all-features --all-targets` passed all 31 E2E tests (and the empty unit-test target). `openspec validate fix-layout-conversion-and-configure-typing-default --strict`, `cargo fmt --check`, and `git diff --check` passed.
