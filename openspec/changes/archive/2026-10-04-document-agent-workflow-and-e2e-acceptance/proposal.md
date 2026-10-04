# Proposal

## Why

The repository has no persistent instructions for coding agents about its OpenSpec workflow or how to verify completed work through terminal-level tests. Record these expectations so future changes follow the same review and acceptance process.

## What Changes

- Add a root `AGENTS.md` requiring every repository change to be tracked in OpenSpec before implementation.
- State that exploration and proposal are planning stages; implementation starts on an explicit request to apply or implement.
- Ask agents to cover user-visible functionality with E2E tests where practical, using passing E2E scenarios as acceptance evidence.
- Document the existing terminal test harness and test command, plus how to handle cases unsuitable for E2E testing.

## Capabilities

### New Capabilities

None. This change documents contributor workflow and does not change application behavior.

### Modified Capabilities

None.

## Impact

Adds only `AGENTS.md`. Existing application code, tests, and user-facing behavior remain unchanged.
