# Spec Delta

## Purpose

Allow terminal typing exercises to interpret Russian and English keys by keyboard position when useful, while letting users keep ordinary typing exercises layout sensitive by default.

## ADDED Requirements

### Requirement: Convert corresponding keys in Translation
The system SHALL enable Russian/English keyboard-position conversion in every Translation exercise, including starts from the Translation menu item, Quick Start configured for Translation, and Select Mode configured for Translation. Conversion SHALL cover every letter of the Russian alphabet, including `ё` and letters whose English-layout keys are punctuation, and SHALL work in both directions where the displayed target uses either layout. A direct input that already matches the displayed target SHALL remain correct.

#### Scenario: Mac semicolon key produces Russian zhe
- **WHEN** a Translation exercise displays `ж` as the next character and the terminal sends `;`
- **THEN** the input is accepted as `ж` and contributes to a correct result

#### Scenario: Every Russian letter is accepted from the English layout
- **WHEN** a Translation exercise displays each Russian letter as the target and the terminal sends the corresponding English-layout key for each letter
- **THEN** each input is accepted as correct, including `ё`, `х`, `ъ`, `ж`, `э`, `б`, and `ю`

#### Scenario: Cyrillic input maps to an English-layout target
- **WHEN** a Translation exercise displays an English-layout target and the terminal sends its corresponding Russian-layout character
- **THEN** each corresponding key is accepted as correct, including targets that are punctuation

#### Scenario: Correct layout remains usable
- **WHEN** a Translation exercise displays a character and the terminal sends that exact character
- **THEN** the character is accepted without a conversion error

### Requirement: Keep ordinary Typing layout sensitive by default
The system SHALL disable automatic Russian/English keyboard-position conversion by default in Typing exercises started through Quick Start or Select Mode. A key from the other layout SHALL count as incorrect unless the user has enabled conversion for Typing in Settings. This choice SHALL also apply when a Typing exercise is restarted.

#### Scenario: Default Typing rejects another-layout key
- **WHEN** a new or existing user starts a Typing exercise with the default setting and enters a corresponding key from the other layout
- **THEN** the result records that key as incorrect

#### Scenario: Default Typing accepts exact key
- **WHEN** a Typing exercise displays a character and the terminal sends that exact character
- **THEN** the character is accepted as correct

### Requirement: Allow persistent conversion default for Typing
Settings SHALL offer a visible on/off option for layout conversion in Typing exercises. The initial and missing-field value SHALL be off. Saving the option SHALL affect later Typing exercises from Quick Start and Select Mode, including restarts, across application launches. The option SHALL NOT disable conversion in Translation exercises.

#### Scenario: User enables conversion for Typing
- **WHEN** a user enables the Typing conversion option in Settings, saves it, and starts a Typing exercise
- **THEN** corresponding keys from the other layout count as correct

#### Scenario: Saved preference survives restart
- **WHEN** a user saves the Typing conversion option, exits, and relaunches the application
- **THEN** Settings shows the saved value and subsequent Typing exercises use it

#### Scenario: Existing settings file lacks the option
- **WHEN** the application loads a settings file created before this option existed
- **THEN** its other settings remain available and Typing conversion is off

#### Scenario: Translation ignores the Typing option
- **WHEN** the Typing conversion option is off and a user starts or restarts a Translation exercise
- **THEN** corresponding keys from the other layout still count as correct
