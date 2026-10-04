# Design

## Context

See `proposal.md` and `specs/keyboard-layout-conversion/spec.md`. `TypingWidget::add_char` currently converts every input based on the next target character. The mapping table contains punctuation-position letters, but `interpret_char_for_target_layout` only enters the English-to-Russian branch for ASCII alphabetic input, so `;` never reaches the `ж` entry. The reverse branch has the same issue when the displayed target is an English punctuation key. `Configuration::Settings` is JSON-backed and existing files do not contain the new option. Exercise widgets are constructed in multiple start paths and again on restart.

## Goals / Non-Goals

**Goals:** Keep one consistent conversion policy per exercise, ensure every construction and restart path passes that policy, and preserve existing settings files.

**Non-Goals:** Detect the operating system keyboard layout, synthesize physical key events, or transliterate words by sound. The terminal provides characters, and the app interprets their keyboard positions.

## Decisions

1. Add a conversion-enabled flag to `TypingWidget`, defaulting to disabled. Resolve it when creating an exercise: always enabled for `TestMode::Translation`, otherwise read a new `Settings` boolean. Preserve the resolved choice when restarting. This keeps mode policy in `App` and character interpretation in the widget. An alternative is reading global settings for every key, which would couple the widget to configuration and allow mid-exercise changes.
2. Replace the ASCII-letter gates with target-aware lookup against the complete Russian/English key-position pairs. Include the punctuation keys `[`, `]`, `;`, `'`, `,`, `.`, and backtick, plus `ё`. Exact input takes precedence. Preserve an unmatched input as-is so mistakes still reduce accuracy. Keep case handling explicit for letter pairs. An alternative is unconditional translation of punctuation, which would corrupt punctuation typed correctly in the target layout.
   The baseline E2E tests also showed that completion currently compares UTF-8 byte lengths. Compare character counts so a mixed Cyrillic/ASCII target finishes after the expected number of characters in either conversion direction.
3. Add one global Typing conversion option to a distinct Typing section of Settings rather than to Quick Start, because the option also applies to Select Mode. Deserialize a missing field as `false` and save the field with existing settings. Translation remains enabled regardless of that setting.
4. Use `tests/e2e_*.rs` and `tests/common/mod.rs` to send the character bytes emitted by a terminal through `script` to `wpm`. Seed deterministic single-entry word and translation sets with `TestDataDir` so result accuracy shows whether each target was interpreted correctly. Start with a narrow `;`/`ж` regression and an exhaustive alphabet fixture, run them against current code, and record the expected failure before editing application code. Then add E2E coverage for default Typing, setting on and persistence, all start paths, and restart. Tests verify terminal-observable results rather than private conversion helpers.

## Risks / Trade-offs

- [Terminals report characters, not physical key positions] → The E2E tests send `;` and the other English-layout characters that a Mac terminal reports for the corresponding keys. A physical Mac keyboard smoke check can supplement the automated test if a specific input source behaves differently.
- [Different target types need distinct conversion directions] → Test both Cyrillic targets and English targets, including punctuation-position letters, and retain direct-input tests.
- [Adding a Settings row changes keyboard navigation] → Update affected existing E2E navigation sequences and assert the visible option and its saved state.

## Migration Plan

On first launch after upgrade, existing settings files deserialize the missing Typing conversion field as off. Saving Settings writes the new field. No database migration is needed.
