# Proposal

## Why

Contributors need a concise map of the current repository and readable tests that protect its actual dependency boundaries. The fetched `origin/master` adds Spanish exercises, dictation/audio, shared language rules and declarative datasets, so the existing architecture tests must be reconciled with those changes and placed in their own directory.

## What Changes

- Integrate the fetched `origin/master` while preserving the current uncommitted architecture work and a recoverable copy through conflict resolution.
- Move the direct architecture rules to `tests/architecture/main.rs`, keeping the Cargo test target named `architecture` and the existing test command.
- Adapt the rules to actual module responsibilities after the merge, including shared settings/models, dictation/audio, language processing and dataset/build-time dependencies.
- Keep the tests as the primary source of exact executable rules; retain ArchUnitRust and the small source-suppression rule without restoring a policy loader or analysis framework.
- Update the concise durable overview and root links to describe the merged architecture and new test location.
- Preserve the requested `AGENTS.md` guidance change and upstream production behavior; do not refactor application modules to satisfy obsolete rules.

## Capabilities

### New Capabilities

- `repository-architecture`: The concise overview and direct architecture tests introduced by this same active change, now reconciled with upstream and organized in a dedicated test directory.

### Modified Capabilities

None outside this change. Upstream feature changes arrive with their own existing OpenSpec artifacts, which this work preserves without rewriting or marking their tasks complete.

## Impact

- The merge brings the fetched upstream source, datasets, tests, build script and OpenSpec artifacts into the branch. Local implementation changes remain scoped to architecture tests/documentation and reconciliation of the development dependency in Cargo files.
- `tests/architecture.rs` moves to `tests/architecture/main.rs`; the root overview pointer and durable spec links are updated.
- Existing Cargo/CI commands discover the relocated target. Full tests cover the combined upstream functionality and local architectural rules.
- The current concise approach remains: no embedded JSON policy, custom inventory, diagnostic exception adapter, Cargo metadata rule engine or permanent generic fixture framework.
