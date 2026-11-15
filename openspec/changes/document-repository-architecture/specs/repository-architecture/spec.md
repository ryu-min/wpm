# Repository Architecture Specification

## Purpose

Give contributors a concise overview of the current repository and point them to readable executable tests that define its architectural dependency rules.

## ADDED Requirements

### Requirement: Maintain an authoritative repository map
The repository SHALL keep a concise current-architecture overview in `openspec/specs/repository-architecture/spec.md`. Exact executable dependency rules SHALL live in `tests/architecture/main.rs` as their primary source of truth, without an embedded specification policy or duplicate selector/allowlist tables.

| Area | Current responsibility |
| --- | --- |
| `src/main.rs`, `src/app.rs` | Terminal lifecycle, application state, events and coordination of screens, configuration and audio. |
| `src/*_widget.rs` | Menu, mode selection, settings, typing, dictation and result rendering/input; widgets use appropriate shared settings or exercise models. |
| `src/configuration.rs` | Settings, SQLite persistence and adapters from bundled datasets to exercise/vocabulary data. |
| `src/dictation.rs`, `src/vocabulary.rs`, `src/language_input.rs` | Dictation session state, vocabulary/translation models and shared language/grapheme rules. |
| `src/audio.rs` | Cancellable audio-process operations and results consumed by application coordination. |
| `src/dataset.rs`, `build.rs`, `resources/datasets/` | Shared dataset validation and build-time discovery/embedding of declarative JSON datasets. |
| `tests/architecture/`, other `tests/` files | Direct architecture rules, dataset checks and terminal acceptance suites. |
| `openspec/`, `.github/`, Cargo files | Specifications/plans, CI and build/dependency configuration. |

```text
main --> app --> widgets / dictation / audio
           |
           +--> configuration --> datasets / vocabulary
shared language rules support typing, vocabulary and dictation
build.rs --> dataset validation --> embedded dataset data
```

Main declares the modules; widgets have selected shared-model dependencies. This describes responsibilities, not an exact dependency graph.

#### Scenario: Contributor finds a responsibility and its rules
- **WHEN** a contributor reads the overview after upstream integration
- **THEN** they can locate the current exercise/audio/dataset responsibilities and the primary rules in `tests/architecture/main.rs`

### Requirement: Provide one stable architecture entry point
Root `ARCHITECTURE.md` SHALL concisely link to the durable overview and `tests/architecture/main.rs`, identifying tests as the source of exact executable rules. It SHALL NOT duplicate the map. `AGENTS.md` SHALL permit durable architectural requirements and SHALL no longer instruct contributors to set `skip_specs: true` for documentation or tooling changes.

#### Scenario: Contributor opens the root architecture document
- **WHEN** a contributor follows its overview and rules links
- **THEN** both resolve to existing files and the executable link targets the dedicated architecture-test directory

### Requirement: Preserve the current dependency boundaries
Architecture tests SHALL use direct named library rules that reflect the merged architecture: screen-specific shared models, persistence/dataset adaptation, session/language/audio responsibilities and entry-point separation. Exact selectors and allowances SHALL live in the tests. The merged production source SHALL pass without refactoring it to satisfy obsolete pre-merge boundaries.

The syntax analyzer includes inline test code. File-level allowances needed for existing inline tests SHALL be explained in the rule source rather than represented as runtime architectural layers. Tests SHALL reject the analyzer suppression marker in selected application and build-script source using a small source-text rule.

#### Scenario: Existing upstream shared dependencies are accepted
- **WHEN** rules analyze settings-consuming widgets, language-aware typing, dictation rendering and configuration's dataset/vocabulary adapters
- **THEN** those legitimate dependencies pass while the tests retain explicit source-specific boundaries

#### Scenario: A supported direct dependency violates a boundary
- **WHEN** an ordinary source reference introduces a prohibited dependency covered by a rule
- **THEN** the named architecture test fails with dependency evidence

#### Scenario: A suppression marker is introduced
- **WHEN** selected production or build-script source contains the reserved suppression marker
- **THEN** the source-text rule fails even when the marker is in a comment or string

### Requirement: Run useful architecture checks with ordinary Cargo tests
The suite SHALL occupy `tests/architecture/` with `main.rs` as the Cargo integration-test entry point. Cargo SHALL discover exactly one integration target named `architecture`; the obsolete `tests/architecture.rs` entry SHALL be absent. Both `cargo test --locked --test architecture` and `cargo test --locked --all-features --all-targets` SHALL execute the rules.

Rules SHALL remain readable without a specification parser or custom inventory/diagnostic/metadata framework. They analyze recognized source dependencies syntactically, including a conservative union of cfg branches. They do not prove full compiler resolution, generated-code dependencies, exhaustive inventory, canonical identity behind Cargo aliases or semantic responsibility correctness. The overview is maintained manually and is not parsed at test runtime.

#### Scenario: The relocated target remains discoverable
- **WHEN** Cargo lists test targets and the developer runs the focused architecture command
- **THEN** one `architecture` target points to `tests/architecture/main.rs` and executes a nonempty suite independently of OpenSpec documents

#### Scenario: Contributor changes structure or dependencies
- **WHEN** a change adds modules, alters shared source/build logic or renames a dependency
- **THEN** acceptance reviews the direct rules and overview and verifies representative forbidden dependencies alongside the existing full regression suite

## REMOVED Requirements

### Requirement: Detect map drift and incomplete analysis
**Reason**: Readable tests are the primary rule source; the earlier exhaustive inventory/diagnostic contract required an unwanted custom framework.
**Migration**: Keep direct architecture tests and their stated syntax limits. The obsolete durable requirement was removed in the earlier simplification and SHALL NOT be restored during merge or synchronization.
