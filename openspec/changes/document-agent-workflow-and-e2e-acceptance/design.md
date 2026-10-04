# Design

## Context

The project already has an OpenSpec root and Unix E2E tests in `tests/e2e_*.rs`. Their shared harness in `tests/common/mod.rs` launches the built `wpm` binary through `script`, sets `WPM_DATA_DIR` to an isolated temporary directory, and captures terminal screens. CI runs `cargo test --locked --all-features --all-targets` on Linux, macOS, and Windows; E2E modules are Unix-only.

## Goals / Non-Goals

**Goals:** Keep agent workflow guidance brief, actionable, and aligned with the existing test setup. Make terminal-level E2E coverage the default acceptance evidence for user-visible changes.

**Non-Goals:** Change the test harness, require E2E tests for purely internal or documentation changes, or add behavioral requirements to main specs.

## Decisions

- Put durable instructions in the root `AGENTS.md`, so agents encounter them when working anywhere in the repository. Keep change-specific acceptance scenarios in each change's `tasks.md`.
- Require an OpenSpec change for repository edits, including documentation and CI. For changes without spec-level behavior, use `skip_specs: true` instead of inventing requirements.
- Tell agents to add or update E2E tests for user-visible functionality where practical. A task is accepted when its relevant E2E scenarios pass. If terminal-level testing is impractical, the change must record why and name an alternative verification method.
- Point to existing helpers and one test command, rather than duplicating harness implementation details in `AGENTS.md`.

## Risks / Trade-offs

- An E2E-first rule may be too broad for internal changes. Limit it to user-visible behavior and require an explicit reason and alternate check where E2E does not fit.
- Tests are Unix-only today. State this clearly so Windows runs are not mistaken for E2E coverage.
