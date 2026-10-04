# Proposal

## Why

The project is still named `wmp` in its Cargo manifest, so a Cargo installation would expose the wrong terminal command. New users also need a short explanation of the app and an installation command they can follow.

## What Changes

- Name the Cargo package and installed executable `wpm` (words per minute).
- Document installation from the existing Git repository using Cargo, plus how to start the app and install from a local checkout.
- Replace the template README with a concise description of typing and translation practice, installation, and license information.
- Update project and test references that identify the executable as `wmp`.
- Check Cargo installation automatically in CI on every pull request and branch push; also check the documented Git installation command after pushes to the repository's default branch.
- **BREAKING:** Rename the optional data-directory override from `WMP_DATA_DIR` to `WPM_DATA_DIR`; the default data location stays the same.

## Capabilities

### New Capabilities

- `local-installation`: Users with Rust and Cargo can install and start the app as `wpm` from the source repository or a local checkout.

### Modified Capabilities

None; there are no existing main specs.

## Impact

Cargo package metadata and lockfile, test references to the executable, the data-directory environment variable, project context, README, and GitHub Actions CI. The Git repository URL remains `https://github.com/ryu-min/wmp`.
