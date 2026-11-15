# Design

## Context

The local work currently has six direct ArchUnitRust tests in `tests/architecture.rs` (83 lines), a concise durable overview, a root pointer, the requested `AGENTS.md` edit and an uncommitted development dependency/lockfile. These edits must survive integration.

Read-only inspection of fetched `origin/master` at `f6dd58b` shows three incoming commits and no unique local branch commits. The new source has 13 modules plus `main.rs`: `app`, `audio`, `configuration`, `dataset`, `dictation`, `dictation_widget`, `language_input`, five earlier widgets and `vocabulary`. `build.rs` shares `src/dataset.rs` and embeds validated `resources/datasets/*.json` through generated output. Upstream includes its own feature/change artifacts and tests.

The old statements that every widget and configuration are internal leaves no longer match the code. Settings/mode-selection widgets consume `configuration::Settings`, typing uses `language_input`, dictation rendering uses session state, and configuration adapts datasets/vocabulary. This plan preserves those responsibilities.

## Goals / Non-Goals

**Goals:** integrate the fetched branch without losing local work; give architecture tests their own discoverable directory; make direct rules accurately describe the combined code; keep exact rules in tests and the spec concise.

**Non-Goals:** application refactoring, invented layers, a restored policy/analysis framework, editing unrelated upstream OpenSpec changes, exhaustive source inventory or compiler-equivalent dependency proof.

## Decisions

### 1. Integrate the branch with recoverable local changes

Before integration, record current HEAD, the fetched target and all tracked/untracked local paths. Save a recoverable copy including untracked architecture documents/tests and the latest planning artifacts, for example an identified `git stash push --include-untracked` entry. Verify the saved entry before changing the branch. Local ignored machine configuration is not part of the tracked change.

The inspected history permits `git merge --ff-only origin/master`; no merge commit or history rewrite is needed. Recheck ancestry and target immediately before integration. Restore saved work with `git stash apply <recorded-id>` rather than discarding the recovery entry while conflicts are unresolved. Reconcile manifests to retain upstream normal/build Unicode dependencies and local `archunit` under development dependencies. Resolve the lockfile using Cargo with the combined manifest, preserving existing resolved versions where possible; avoid an unrelated full dependency update.

Verify upstream source/resources/build/tests and other changes remain present, then compare restored architecture work with the saved snapshot. Keep the recovery reference until restoration and acceptance are verified. If ancestry or local state differs from the inspected situation, inspect that difference before choosing another merge strategy; never force-reset away work.

### 2. Use the conventional directory test target

Move the suite to `tests/architecture/main.rs`. Cargo discovers this directory-entry layout as the integration target `architecture`; preserve `cargo test --locked --test architecture` and the existing CI full-suite command. Remove the old `tests/architecture.rs` so there is only one target. Keep the suite in one readable file unless a concrete need for a small module appears; the directory is not permission to distribute a framework among helpers.

Use `CARGO_MANIFEST_DIR` to locate the application as before. Verify discovery after the move with Cargo metadata and `cargo test --locked --test architecture -- --list`: exactly one target must point to the new entry, with a nonempty rule list. No explicit `[[test]]` entry or CI edit is necessary for this conventional layout.

### 3. Protect the actual source-file boundaries

Keep direct fluent ArchUnitRust tests and strict default nonempty selectors. Narrow allowances by module responsibility rather than granting every widget access to all shared modules. Small loops over a few named files are acceptable when the complete rule remains visible in the same test; no serialized policy or generic evaluator is needed.

The observed internal dependencies guide the rules:

| Source/group | Internal allowance or constraint |
| --- | --- |
| `menu_widget`, `result_widget` | No other source-module dependencies. |
| `settings_widget`, `mode_select_widget` | `configuration`, for shared Settings data. |
| `typing_widget` | `language_input`. |
| `dictation_widget` | `dictation` session/view types. |
| `configuration` | `dataset` and `vocabulary`; no application, screen or audio dependency. |
| `dictation` | `language_input` and `vocabulary`; no terminal UI, persistence or audio-process dependency. |
| `vocabulary` | Runtime uses `language_input`; its existing inline tests also reference `dictation` and `typing_widget`. |
| `audio`, `language_input`, `dataset` | No dependency on other application source modules. |
| `app` | Coordinates screens/services; does not depend on `main`. |
| `build.rs` | Uses `src/dataset.rs`; no dependency on runtime coordination, storage, audio or screens. |

The library is file-based and conservatively includes inline `#[cfg(test)]` code. Therefore document the narrow `vocabulary -> dictation/typing_widget` allowance as evidence from existing inline tests, not a claimed runtime dependency. Similarly, configuration's external `unicode_normalization` import exists in its inline tests. Do not move these tests, ignore their files, suppress evidence or add a custom cfg parser. File-level allowance for `configuration::Settings` does not prove that a widget cannot call other configuration APIs; semantic ownership remains a review concern.

Preserve/readjust external allowlists from actual imports: widgets use standard/terminal crates, with `unicode_normalization` specifically for typing; configuration uses standard/persistence crates plus its inline-test normalization dependency. The new helper modules should remain free of UI/persistence dependencies except their observed needs: `audio` and `dictation` use standard-library externals; `language_input` uses Unicode normalization/segmentation; `dataset` uses Serde/JSON/normalization; `vocabulary` uses Serde. `build.rs` directly uses standard library and Serde JSON. Use Cargo-visible names, as before; exact allowances live only in the tests.

Build and runtime both include `dataset.rs`; verify the analyzer handles this observed source sharing without dropping either context. Main's module declarations remain valid. Internal self-edges are excluded by the library. Do not make the diagram or graph a required exact snapshot.

### 4. Keep the small suppression rule and honest limits

Continue using one built-in file-content predicate rejecting the reserved `archunit:` marker; cover selected `src/` files and the newly present `build.rs`. It must use the library's file/content API rather than custom filesystem traversal or a directive parser.

The suite remains syntax-based. It does not expand macros or the `include!` of generated dataset data, distinguish individual types within a dependency file, prove complete source inventory, resolve every diagnostic or recover canonical package identity behind arbitrary Cargo renames. Inline cfg branches contribute to the observed file graph. Review new modules, source sharing, manifest aliases and responsibility changes alongside the rules. Preserve these limits in a short test comment and the overview.

### 5. Update the concise map and verify useful failures

The durable overview describes orchestration, screen widgets, shared language/session/vocabulary models, audio processes, persistence and the dataset build pipeline. It points to `tests/architecture/main.rs` for exact rules; it contains no dependency policy matrix. Root `ARCHITECTURE.md` links the overview and relocated entry. Preserve the existing `AGENTS.md` edit and leave unrelated incoming OpenSpec artifacts intact.

Run the actual relocated suite against the merged project first to establish that legitimate upstream edges pass. In temporary isolated copies, exercise a small set of important forbidden edges: a widget reaching a sibling or audio service, configuration reaching app, dictation reaching audio, a leaf/shared parser reaching a screen, and a forbidden external crate; include a suppression-marked import. Keep the current settings/model, Unicode, build-to-dataset and inline-test allowances passing. Each negative check must reach and fail the intended named architecture rule, not only fail compilation. Record results rather than adding a general fixture framework.

Run the required full Cargo test command, including the incoming dataset/Spanish suites, plus formatting, Clippy, link/scope checks and strict validation of this change and durable specs. No manual audio check is added for an architecture-test relocation: upstream runtime/audio behavior is preserved and existing automated acceptance provides regression evidence. Report platform coverage accurately.

## Risks / Trade-offs

- Cargo manifest/lock conflicts can discard one side -> Keep a recovery reference, combine dependency intent and let Cargo reconcile the final lockfile.
- Old leaf rules reject legitimate upstream sharing -> Use the observed per-module allowances and verify positive edges before testing negatives.
- File-level cfg union sees test-only dependencies -> Explain narrow allowances for inline tests, keeping semantic runtime claims separate.
- Build-time generated input is outside syntax expansion -> Cover the source-level build/parser boundary and rely on incoming build/dataset tests for data behavior.
- Directory move can silently drop discovery or leave duplicate targets -> Check Cargo metadata and the named target's rule list.
- Growing module count can encourage overengineering -> Keep directly readable rules and brief documentation; no metadata/inventory/parser framework.

## Migration Plan

Save and verify recoverable local work, fast-forward to the fetched upstream revision, restore/reconcile local changes, relocate and adapt the direct suite, and sync the concise overview and links. Complete targeted and full acceptance before considering the combined work finished. Prior completed tasks/evidence remain history; the new merge/adaptation tasks govern this round. No user-data migration or unrelated production refactor is required.
