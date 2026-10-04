# Proposal

## Why

Automatic Russian/English keyboard layout conversion fails for `ж` when a Mac terminal sends `;`, despite the mapping being present. Conversion also runs in ordinary typing exercises where users may want to practice typing in the displayed layout.

## What Changes

- Correct layout conversion for all Russian alphabet keys, including letters reached through punctuation keys such as `;` for `ж`.
- Keep conversion enabled for Translation exercises, including Translation started through Quick Start or Select Mode.
- Disable conversion by default for ordinary Typing exercises and add a saved Settings option to enable it for those exercises, including Quick Start and Select Mode.
- Add terminal E2E coverage that first reproduces the `ж` failure and checks the rest of the Russian alphabet, then covers mode defaults, the setting, and its persistence.

## Capabilities

### New Capabilities

- `keyboard-layout-conversion`: Russian/English keyboard-position conversion during exercises and the setting that controls it for Typing.

### Modified Capabilities

None.

## Impact

The change affects `src/typing_widget.rs`, exercise setup and restart paths in `src/app.rs`, the saved settings in `src/configuration.rs`, the Settings UI in `src/settings_widget.rs`, and terminal E2E tests in `tests/`. Existing settings files must load with the new Typing default.
