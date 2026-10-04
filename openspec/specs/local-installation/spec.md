# Local Installation Specification

## Purpose

Make the terminal application available as the `wpm` command to users who have Rust and Cargo, with clear instructions for installation and startup.

## Requirements

### Requirement: Install from the source repository
The application SHALL be installable with Cargo from its existing Git repository, and the installed executable SHALL be named `wpm`.

#### Scenario: Install from Git
- **WHEN** a user with Rust and Cargo installs the project from `https://github.com/ryu-min/wmp` using the documented Cargo command
- **THEN** Cargo installs an executable named `wpm` that can start the terminal application

### Requirement: Install from a local checkout
The application SHALL be installable with Cargo from a local project checkout under the executable name `wpm`.

#### Scenario: Install from checkout
- **WHEN** a user runs the documented local Cargo installation command in the project directory
- **THEN** Cargo installs an executable named `wpm` that can start the terminal application

### Requirement: Verify installation in CI
The project SHALL automatically verify installation of the current checkout in an isolated Cargo root on every pull request and branch push. On pushes to the repository's default branch, it SHALL also verify the documented installation command against the public Git repository. CI SHALL fail when installation fails or the expected `wpm` executable is absent.

#### Scenario: Pull request or branch push
- **WHEN** CI runs for a pull request or branch push
- **THEN** it installs the checked-out project with Cargo and verifies that the installation root contains the `wpm` executable (`wpm.exe` on Windows)

#### Scenario: Default branch push
- **WHEN** CI runs after a push to the repository's default branch
- **THEN** it additionally runs the documented Git installation command and verifies that the installation root contains the `wpm` executable (`wpm.exe` on Windows)

### Requirement: Document installation and purpose
The README SHALL concisely describe the application's typing and Russian-to-English translation practice, show how to install it with Cargo, identify the Cargo prerequisite and binary-directory PATH requirement, and show the `wpm` launch command.

#### Scenario: New user follows the README
- **WHEN** a new user reads the README
- **THEN** they can identify what the app does and the commands and prerequisites needed to install and launch `wpm`

### Requirement: Override the user data directory
The application SHALL use `WPM_DATA_DIR` as its optional user data-directory override and SHALL otherwise continue to use its existing default data location.

#### Scenario: Override is set
- **WHEN** `WPM_DATA_DIR` points to a directory and the user starts `wpm`
- **THEN** the application uses that directory for its settings and database

#### Scenario: Override is absent
- **WHEN** `WPM_DATA_DIR` is absent and the user starts `wpm`
- **THEN** the application uses its existing default data location
