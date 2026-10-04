# Tasks

## 1. Agent instructions

- [x] 1.1 Add root `AGENTS.md` with the OpenSpec workflow boundary for all repository edits; verify it covers planning, explicit implementation handoff, and `skip_specs: true` for changes without behavioral requirements.
- [x] 1.2 Document E2E tests as the preferred acceptance check for user-visible behavior, including existing Unix PTY helpers, isolated test data, the test command, and a documented alternative when E2E is impractical; verify the guidance matches `tests/common/mod.rs` and CI.

## 2. Integration check

- [x] 2.1 Validate the OpenSpec change and inspect the `AGENTS.md` diff for scope and consistency; verify `openspec validate document-agent-workflow-and-e2e-acceptance` succeeds.
