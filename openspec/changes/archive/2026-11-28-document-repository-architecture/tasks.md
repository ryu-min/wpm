# Tasks

## Согласование с remove-dictation — 16 ноября 2026

Активные proposal/design/specs согласованы с [remove-dictation](../remove-dictation/proposal.md). Ниже сохранены исторические задачи, их фактические отметки и результаты прежних раундов; они не являются поручением повторно вводить отменённое поведение. Текущая реализация и регрессионная приёмка выполняются по [задачам remove-dictation](../remove-dictation/tasks.md). Упоминания прежних записей об удалённых аудиоспецификациях и старых путях кода относятся к тому историческому состоянию; актуальные требования доступны в оставшихся specs этого change и remove-dictation.

Capability repository-architecture уже опубликована: будущая delta использует MODIFIED. Прежнее удаление Detect map drift and incomplete analysis уже отражено в основной спецификации; повторная REMOVED-операция к отсутствующему требованию исключена.

The user requested tests as the primary source of exact rules, then requested upstream integration and a dedicated test directory. Groups 1–5 describe superseded machinery; group 6 records the completed direct-rule rewrite. Group 7 is the active merge/relocation/adaptation scope. Preserve the earlier evidence as history, not as acceptance for the newly merged code.

## 1. Validate and select the dependency analyzer

- [x] 1.1 Add the candidate `archunit` development dependency and resolve `Cargo.lock` through Cargo; verify a minimal integration check compiles on the project's stable toolchain without changing production source.
- [x] 1.2 Run the bounded compatibility check from `design.md` against the real binary target and isolated fixture sources; verify all eight production files, structural main declarations, known app dependencies, imports/aliases/relative paths, qualified references and forbidden external crates are discovered, and record the library/version and outcomes here. Retain useful fixtures for the final suite.
- [x] 1.3 Confirm the candidate satisfies the matrix, or evaluate Archaven against that same matrix and document a justified substitution in `design.md`; completion requires recorded evidence for the chosen library, with no silently weakened requirements or custom Rust analyzer.
- [x] 1.4 Verify the bounded `UnknownReference` exception against the library's public dependency-reference data: every accepted production occurrence must have the reviewed source/head, empty candidates and an unresolved `PathReference` match; record that aliases, type aliases and foreign impl/`Self` probes retain forbidden edges and all other diagnostic classes remain failures.

## 2. Publish the architecture contract and entry point

- [x] 2.1 Sync `specs/repository-architecture/spec.md` from this change into `openspec/specs/repository-architecture/spec.md` through the OpenSpec sync workflow; verify the durable file contains its Purpose, all requirements/scenarios, source/supporting-area maps, diagram and exactly one intact JSON policy block, and validate the durable specs.
- [x] 2.2 Add concise root `ARCHITECTURE.md` identifying the durable spec as the current architecture source; verify its relative link resolves to that existing file and the entry point does not duplicate the map or policy.
- [x] 2.3 Revise the targeted OpenSpec bullet in `AGENTS.md` to permit architectural requirements and remove the instruction to set `skip_specs: true`; verify the resulting guidance preserves the existing proposal/apply and acceptance workflow and contains no such opt-out instruction.

## 3. Connect the specification to executable architecture rules

- [x] 3.1 Add the small fixed-marker JSON loader and policy validation to `tests/architecture.rs` or a dedicated architecture-test helper; verify focused tests reject missing/duplicate/malformed blocks, ambiguous JSON, invalid paths and unknown targets, and load the actual durable policy successfully with no separate production allowlist.
- [x] 3.2 Add source inventory checks comparing the policy, recursive `src/**/*.rs` enumeration and the analyzer's production files; verify the current eight files match and isolated new orphan, missing/renamed, empty-analysis and unclassified-node cases fail with identifiable paths.
- [x] 3.3 Translate the loaded per-file policy into the selected library's internal and external dependency rules; verify the current project passes, main's seven structural declarations remain valid, and relevant analysis failures cannot be mistaken for a successful check.
- [x] 3.4 Implement the reviewed per-file `unresolved_type_heads` diagnostic gate without filtering resolved edges; verify existing associated paths pass, new uppercase/lowercase heads and mismatched-file or import/macro occurrences fail, and forbidden uppercase aliases/reexports, type aliases and foreign impl/`Self` cases still fail the dependency rules through the shared production evaluation path.
- [x] 3.5 Complete representative isolated fixtures using the same policy-evaluation functions as production; verify forbidden widget-to-app/configuration/widget, configuration-to-app/widget/UI-crate and widget-to-storage-crate references fail, including grouped/aliased/relative imports and qualified type/expression paths, while allowed references and comments/strings pass.
- [x] 3.6 Verify the durable specification accurately documents the implemented focused/full Cargo commands, source-analysis limitations and manual responsibility review; run `cargo test --locked --test architecture` and record the result, including successful expected-violation fixture assertions.

## 4. Verify integration and completion

- [x] 4.1 Run `cargo test --locked --all-features --all-targets` on a Unix host and record the result; verify the existing terminal E2E suites still pass alongside architecture tests. The change adds no terminal behavior, and terminal execution cannot prove source dependency boundaries, so focused source/fixture tests provide architectural acceptance while E2E provides behavioral regression coverage.
- [x] 4.2 Run `cargo fmt -- --check` and `cargo clippy --locked --all-features --all-targets`; record results and verify the new suite is not Unix-gated and is included by the existing Linux/macOS/Windows CI test command. Report unexecuted platform checks accurately; do not claim local Windows coverage.
- [x] 4.3 Run strict OpenSpec validation for this change and the durable specs, inspect the final diff, and verify the durable link/policy work without reading the change directory; completion requires all tasks checked, no changes to production source/resources or existing behavior requirements, and only the intended documentation/test/development-dependency changes.

## 5. Close dependency-analysis gaps

- [x] 5.1 Map Cargo-visible external targets to original package names using the current package's normal-dependency metadata, including conservative target-specific alias sets; verify through the shared evaluator that a widget using Cargo-renamed `serde_json` fails with source/alias/package evidence, configuration using the same alias passes, and unavailable/invalid metadata or an unmapped non-sysroot external target fails explicitly.
- [x] 5.2 Reject recognized analyzer suppression directives in all production source before extraction using the bounded line-text grammar in `design.md`; verify shared-evaluator fixtures for trailing, preceding, scoped, whitespace/CRLF and suppressed-unresolved imports fail with source and line, while ordinary text and non-directive lookalikes pass preflight.
- [x] 5.3 Sync the updated package-identity and suppression requirements/scenarios into the existing durable architecture specification through the OpenSpec sync workflow; verify the executable policy block and prior map/requirements are preserved, the root link still resolves and strict validation of this change and durable specs passes.
- [x] 5.4 Run the focused architecture suite normally and with offline Cargo, then `cargo test --locked --all-features --all-targets`, formatting, Clippy and diff checks; record the new results and verify the fixes change only adapter/tests and the planned architectural documentation, preserving production source/resources and per-file dependency allowances.

## 6. Make direct tests the primary architectural rules

- [x] 6.1 Rewrite `tests/architecture.rs` as readable ArchUnitRust fluent rules for independent widgets, independent configuration, app-to-main separation and the observed UI/persistence external allowlists; keep default nonempty-selector handling and verify current production passes `cargo test --locked --test architecture` without refactoring.
- [x] 6.2 Add one small built-in source-text predicate rejecting `archunit:` in production files and remove the old JSON/spec loader, inventory/diagnostic/meta adapters and fixture framework; verify tests read no OpenSpec document and equivalent machinery was not moved to helpers. Briefly document syntax/Cargo-name limitations beside the rules.
- [x] 6.3 Run actual rewritten tests in isolated temporary copies with representative forbidden widget/configuration/app and external dependencies, ordinary/qualified references and a suppression-marked import; record failure of the intended rule with evidence rather than compilation failure. Verify current main declarations pass; discard mutations and add no permanent general fixture framework.
- [x] 6.4 Sync the concise delta into the durable spec, removing the obsolete `Detect map drift and incomplete analysis` requirement and embedded policy/adapter details, and update root `ARCHITECTURE.md` with links to overview and primary test rules; verify links, source-of-truth wording and strict OpenSpec validation. Preserve the requested `AGENTS.md` change.
- [x] 6.5 Run `cargo test --locked --all-features --all-targets`, `cargo fmt -- --check`, `cargo clippy --locked --all-features --all-targets` and diff/OpenSpec checks; record results and compare total architecture-test/helper code with the 937-line starting point. Verify substantial simplification through deletion/direct readability, no displaced framework or arbitrary line-count test, and unchanged production/resources/behavior.

## 7. Integrate upstream and organize the adapted architecture suite

- [x] 7.1 Record current HEAD, fetched upstream revision and tracked/untracked local changes; create and verify a recoverable snapshot/stash including untracked architecture artifacts, fast-forward to the inspected upstream, and restore local work without discarding its recovery reference during conflicts. Reconcile Cargo manifests/lockfile to retain both upstream normal/build dependencies and local `archunit`; verify upstream ancestry/content and preservation of saved local work and unrelated incoming OpenSpec artifacts.
- [x] 7.2 Move `tests/architecture.rs` to `tests/architecture/main.rs`; verify the old entry is absent, Cargo metadata reports exactly one `architecture` test target at the new path, and `cargo test --locked --test architecture -- --list` lists nonempty rules without CI/configuration changes or a helper framework.
- [x] 7.3 Adapt direct rules to the observed widget/model, configuration/dataset/vocabulary, dictation/language/audio and build/parser boundaries; document narrow inline-test allowances and include the build script in suppression coverage. Verify the merged source passes the focused architecture suite, including legitimate settings-model, Unicode, shared-dataset and cfg-test dependencies; do not disable rules or refactor production source to make them pass.
- [x] 7.4 In isolated temporary copies, run actual relocated rules against representative forbidden widget/sibling/audio, configuration/app, dictation/audio, parser/screen and external-crate dependencies plus a suppression-marked import. Verify intended rule failures with evidence rather than compilation failures, record results, and discard the mutations without retaining a general fixture generator.
- [x] 7.5 Sync the updated concise overview to the durable spec and update root links to `tests/architecture/main.rs`; verify the map reflects incoming audio/dictation/language/dataset/build responsibilities, exact rules remain in tests, obsolete JSON/inventory details stay removed and strict change/durable-spec validation passes. Preserve the requested AGENTS edit and unrelated upstream OpenSpec artifacts/tasks.
- [x] 7.6 Run `cargo test --locked --all-features --all-targets` including the incoming dataset/Spanish acceptance suites, formatting, Clippy, whitespace and final link/scope checks; record commands/results/platform limits. Verify production source/resources/build logic match the integrated upstream rather than accidental local conflict edits, the local architecture work remains present and the test target still runs through existing CI commands.

## Compatibility evidence (superseded implementation)

The evidence below records the first implementation pass. Group 5 tracks additional acceptance for Cargo package aliases and analyzer suppression; its results must be recorded before treating the revised contract as complete.

- ArchUnitRust (`archunit` 0.0.2) compiled on Rust 1.95.0 stable. Production probe discovers eight source self-markers, all seven `main` module declarations and all six application dependencies.
- All 133 production `UnknownReference` diagnostics match unresolved `PathReference` records by source, subject and line. Reviewed heads are encoded per source in the specification; no graph edges are suppressed.
- Seven isolated probes retain the forbidden internal/external edge for imported type aliases, uppercase external crate/item aliases, uppercase module aliases/reexports, a local type alias, and a foreign impl using `Self`. The latter two additionally emit the expected unresolved associated-path diagnostic.
- Local macOS default SDK 27.0 is incompatible with the installed linker (`arm64e.x1` TBD architecture). Verification uses the already installed compatible SDK with `SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk`; no repository/toolchain configuration is changed.

- Compatibility matrix completed through the shared production evaluator: ordinary/grouped/relative imports, module/item aliases, type/expression qualified paths, every listed external crate, retained structural edges, comments/strings, malformed policies and inventory failures all exercised. ArchUnitRust satisfies the clarified contract; Archaven fallback is unnecessary.
- Synced the new capability via the OpenSpec sync workflow (new main spec, Purpose preserved, ADDED requirements merged under Requirements). Strict durable-spec validation passes; exactly one unchanged policy block and a resolving root pointer were verified.

- Focused `cargo test --locked --test architecture`: 8 passed, including expected dependency-rule violations and fail-closed diagnostic/policy/inventory assertions. The suite also accepts CRLF documents for Windows checkouts. The durable spec records focused/full commands, conservative cfg analysis and manual map/exception review.

## Final verification (superseded implementation)

- macOS / Unix: `cargo test --locked --all-features --all-targets` passed (8 architecture tests and 31 terminal E2E tests across six existing suites; binary unit target has no tests). Source/fixture assertions provide architecture acceptance because terminal execution cannot prove source dependency boundaries; E2E supplies behavior regression coverage.
- `CARGO_NET_OFFLINE=true cargo test --locked --test architecture` passed: 8/8, proving checks need no network service once dependencies are available. Built-in dependency violations are asserted specifically, separately from rejected analyzer diagnostics.
- `cargo fmt -- --check`, `cargo clippy --locked --all-features --all-targets`, `git diff --check`, strict validation of this change and all three durable specs passed. OpenSpec emits only informational long-requirement notices.
- Cargo commands requiring linking used the SDKROOT override recorded above. Linux and Windows were not executed locally. The architecture target has no Unix cfg gate; existing CI includes all targets on Linux, macOS and Windows. CRLF policy loading is explicitly tested.
- Final scope: development dependency and Cargo-generated lockfile; source/fixture tests; durable spec; root pointer; the targeted AGENTS bullet; change artifacts/evidence. Production `src/`, `resources/`, existing behavior specs and CI are unchanged. Runtime policy loading uses only the durable spec, never this change directory. Main/app dependency evidence is tested with explicit passing fixtures so the production policy remains an allowlist rather than requiring currently unused edges to persist.

## Review round 1 fixes

- Cargo metadata is collected offline and selected by canonical current manifest path. Normal dependencies map normalized visible names to package-name sets, retaining target-specific declarations. External checks deny a source when any alias identity is forbidden, preserve permitted-owner scope, and reject unmapped non-sysroot targets. Fixtures cover Cargo aliases (`storage`, `data-store`), the same alias in configuration, a target-dependent alias pointing to allowed/forbidden package identities, dev/build exclusion and metadata failure/invalid selection paths.
- Preflight checks every enumerated production file before either analyzer extraction. Its bounded physical-line grammar mirrors ArchUnitRust 0.0.2's ignore directive prefix, whitespace and word-boundary checks. Trailing, preceding, scoped, whitespace/CRLF, unresolved-import, recognizable string/block-comment and unattached directives fail with path/line; lookalikes pass preflight and still obey dependency rules.
- Updated dependency-identity and suppression requirement bodies/scenarios are synced into the existing durable spec; all five prior requirements, map, Purpose and JSON policy remain present. No package or file allowances changed.

- Round 2 verification: focused architecture suite passed normally and with `CARGO_NET_OFFLINE=true` (11/11). Full `cargo test --locked --all-features --all-targets` passed (11 architecture + 31 terminal E2E tests). Formatting, Clippy, whitespace and strict change/all-durable-spec checks passed. The shared-evaluator missing-metadata fixture also passed after tightening its assertion path.
- Local SDKROOT workaround remains as recorded for the initial round. Linux/Windows were not executed locally. Review fixes add no dependency, production source/resource/CI edit or per-file allowance change; runtime policy still loads only the durable spec.

## Direct-rule rewrite evidence

- Replaced the superseded 937-line adapter/fixture suite with six directly named fluent ArchUnitRust rules in `tests/architecture.rs` (83 lines including explanatory comments). All six pass against unchanged production source. No OpenSpec reads, policy model, inventory, diagnostic gate, metadata adapter, temporary-fixture generator or displaced helper remain.
- The durable overview now contains four concise requirements and its scope-appropriate Purpose, with the obsolete map-drift/incomplete-analysis requirement removed. Root `ARCHITECTURE.md` links both the overview and primary executable tests. The spec explicitly records syntax, generated-code, inventory and Cargo-visible-name limitations; exact selectors/allowances live in tests.

- Seven isolated temporary project copies ran the actual rewritten `tests/architecture.rs` with `cargo test --locked --test architecture <named-test> -- --exact` (offline Cargo, shared dependency build cache, each integration test rebuilt for its temporary manifest). Each mutation compiled successfully and failed its intended named rule with the source/target shown below. The copies contained no OpenSpec documents and were discarded; no mutation helper or fixture framework is retained in the repository. Baseline production passes all six rules, including main's existing module declarations.

| Temporary mutation | Rejected by rule | Evidence |
| --- | --- | --- |
| Widget imports configuration settings | `widgets_are_independent_of_other_production_modules` | `src/menu_widget.rs` -> `src/configuration.rs` |
| Widget uses qualified sibling type | `widgets_are_independent_of_other_production_modules` | `src/menu_widget.rs` -> `src/typing_widget.rs` |
| Configuration imports application | `configuration_is_independent_of_other_production_modules` | `src/configuration.rs` -> `src/app.rs` |
| Application uses qualified entry-point function | `application_does_not_depend_on_the_entry_point` | `src/app.rs` -> `src/main.rs` |
| Widget uses qualified JSON type | `widgets_use_only_terminal_ui_dependencies` | `src/menu_widget.rs` -> `serde_json` |
| Configuration imports terminal frame | `configuration_uses_only_persistence_dependencies` | `src/configuration.rs` -> `ratatui` |
| Widget import has analyzer suppression directive | `production_source_does_not_suppress_architecture_analysis` | `src/menu_widget.rs` fails reserved-marker predicate |

- Rewrite verification: `cargo test --locked --test architecture` passed all 6 direct rules; offline focused execution also passed. After temporary mutation experiments, package-only build artifacts were cleared and `cargo test --locked --all-features --all-targets` passed freshly against the actual repository (6 architecture + 31 terminal E2E tests, no compiler warnings). Formatting, Clippy, whitespace checks and strict validation of the change/all durable specs passed.
- Architecture-test/helper implementation is now 83 lines total versus 937: 854 lines removed (about 91%). There are no architecture helpers or committed fixtures. The durable overview is 65 lines. Reduction comes from deleting the policy/adapter/fixture machinery; normal formatting and six individually named direct rules remain.
- Production source/resources, existing behavior specs, CI and the requested AGENTS wording are unchanged by this rewrite. Test runtime does not read OpenSpec. Limits are documented beside the tests and in the overview: supported syntactic dependencies only, Cargo-visible external names, manual review for aliases/new modules/responsibilities, no exhaustive inventory or diagnostic guarantee. SDKROOT workaround remains necessary locally; Linux/Windows were not executed here.

## Upstream integration and dedicated suite

- Re-fetched `origin/master`, verified 0 unique local commits / 3 incoming commits and ancestor relationship. Fast-forwarded `c8eba5a0226c27d72159bb93b1edfc0599efddfc` to `f6dd58be9f03286528586fd2b7ff152b0e3d14b6`.
- Recovery stash `3f0ab491eecf6b3660cf2b4951ebc944625be563` (`architecture-before-master-f6dd58b-2026-11-15`) retains all 11 pre-integration tracked/untracked paths, verified against pre-stash content hashes. Applied rather than popped it; recovery remains available. Only `Cargo.lock` conflicted. Resolved from the upstream lock through Cargo's offline dependency resolution; every upstream package/version remains, with the local development analyzer dependencies added. Restored the nine non-Cargo local files byte-for-byte before the planned relocation/documentation edits. Combined manifest differs from upstream only by `archunit` under dev-dependencies.
- Cargo metadata reports exactly one `architecture` target at `tests/architecture/main.rs`; the old `tests/architecture.rs` is absent. `cargo test --locked --test architecture -- --list` lists 12 named rules; focused execution passes all 12 against integrated upstream source, including settings-model, Unicode, dictation and shared dataset/build dependencies. Rules remain direct and file-specific; comments explain vocabulary/configuration allowances caused by inline cfg-test code.

- Synced the concise overview to current upstream audio/dictation/language/vocabulary/dataset/build responsibilities, keeping exact rules only in `tests/architecture/main.rs`. Updated root executable link; both overview/rules links resolve. The obsolete JSON/inventory/diagnostic contract remains absent. Only this change's planning/task artifacts and architecture spec were edited; unrelated upstream changes/tasks remain as integrated.

- A one-off analyzer graph check confirmed both `build.rs -> src/dataset.rs` and `src/configuration.rs -> src/dataset.rs` survive shared-source discovery. No graph-check helper was added to the repository.
- Eight isolated temporary copies ran the actual relocated target with `cargo test --locked --test architecture <named-rule> -- --exact`, offline Cargo and a rebuilt integration target for each manifest. All mutations compiled, then failed the intended rule with source/target evidence; temporary copies were discarded and no generator/helper was committed:

| Mutation | Intended rule | Evidence |
| --- | --- | --- |
| Widget -> sibling | `leaf_modules_do_not_depend_on_other_source_files` | `src/menu_widget.rs` -> `src/typing_widget.rs` |
| Settings widget -> audio | `widgets_depend_only_on_their_shared_models` | `src/settings_widget.rs` -> `src/audio.rs` |
| Configuration -> app | `configuration_depends_only_on_dataset_and_vocabulary` | `src/configuration.rs` -> `src/app.rs` |
| Dictation -> audio | `dictation_depends_only_on_language_and_vocabulary` | `src/dictation.rs` -> `src/audio.rs` |
| Shared language parser -> screen | `leaf_modules_do_not_depend_on_other_source_files` | `src/language_input.rs` -> `src/menu_widget.rs` |
| Session -> JSON crate | `support_modules_keep_external_dependencies_small` | `src/dictation.rs` -> `serde_json` |
| Suppression-marked widget import | `production_and_build_source_do_not_suppress_architecture_analysis` | `src/menu_widget.rs` |
| Build-script suppression marker | `production_and_build_source_do_not_suppress_architecture_analysis` | `build.rs` |

- Final merged verification: `cargo test --locked --all-features --all-targets` passed all 94 tests: 18 binary unit tests, 12 architecture rules, 2 dataset-format tests and 62 terminal E2E tests, including incoming dataset-profile/catalog/Spanish translation/dictation/settings coverage. Focused `cargo test --locked --test architecture` also passed offline after temporary-mutation build artifacts were cleared. Cargo target discovery/listing remains unchanged except its new directory entry.
- `cargo fmt -- --check`, `git diff --check`, strict selected-change/all-durable-spec validation and `cargo clippy --locked --all-features --all-targets` passed. Clippy reports existing upstream warnings in unchanged `build.rs`, `tests/common/mod.rs` and `tests/e2e_dataset_profiles.rs` (collapsible conditionals, single-character append, type complexity and needless lifetime); architecture rules add no warnings. No unrelated lint edits were made.
- Final scope checks confirm production `src/`, resources, build logic, incoming tests/helpers/scripts/docs/CI and unrelated OpenSpec artifacts match integrated HEAD. The retained local changes are architecture rules/docs, the requested AGENTS bullet and the development dependency/Cargo-generated lock additions. Direct rules occupy one 196-line file with no helper framework; 12 named tests protect the expanded upstream module set. Recovery stash remains available for review.
- Verification ran on macOS/Unix with the previously documented compatible SDKROOT override. Linux/Windows were not run locally. Existing CI's all-targets command discovers the relocated suite; no CI or explicit test-target configuration was added. No manual audio test or archive was performed for this architecture relocation.
