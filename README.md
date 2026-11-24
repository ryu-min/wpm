# wpm

A terminal app for practicing typing and Russian-to-English translation. Choose
a word set and time limit, then see your speed in words per minute (WPM) and
accuracy. Settings are stored locally.

## Install

Requires [Rust and Cargo](https://www.rust-lang.org/tools/install). Install from
the source repository:

```sh
cargo install --git https://github.com/ryu-min/wmp --locked
```

Make sure Cargo's binary directory (usually `~/.cargo/bin` on macOS and Linux)
is in your `PATH`, then run:

```sh
wpm
```

To install from a local checkout instead, run `cargo install --path . --locked`
in the project directory.

## License

MIT. See [LICENSE](LICENSE).

## Spanish practice

`Translation` includes Russian–English and Russian–Spanish sets. Spanish sets
`ru_es_a1` through `ru_es_c2` and the 31-entry `ru_es_cafe_beginner` set are available. These are application
vocabulary levels, not a certified CEFR course. Translation types both sides of
each pair; Enter types the line break between pairs. Backspace can cross it.


Spanish works with ENG and RU keyboard layouts. The same physical RU keys are
accepted for ENG letters (`рщдф` → `hola`; this is not transliteration). Spanish
comparison accepts `n/ñ`, `a/á`, `e/é`, `i/í`, `o/ó`, and `u/ú/ü`, preserving
case by default. For example, `ano` is accepted for `año` and `si` for `sí`; this convenience
does not test knowledge of accents. Canonical spelling is always shown in the
exercise. Exact Unicode input and composed accents work too.
The ordinary Typing layout-conversion setting remains independent.

The two exercise modes are Typing and Translation. Quick Start uses your saved
mode, dataset, and duration; Select Mode lets you choose them for one session.
Settings has Quick Start, Translation, and Typing sections. Enter saves settings,
Escape discards edits. Escape returns from an exercise to the menu; Ctrl+C quits.

Settings has separate `Ignore case: On/Off` switches for `Typing` and
`Translation`, both Off by default. Left/Right changes the selected switch;
Enter saves it and Escape discards edits. Each mode uses its saved switch in
Quick Start, Select Mode, and Restart; direct Translation uses it too.
With On, letter case is optional throughout the full translation pair, on both
sides, while canonical spelling stays visible. This supplements the existing
layout and Spanish accent rules; Typing layout conversion remains independent.
With Off, the usual strict case rules apply.

When upgrading from a version with Dictation selected for Quick Start, it becomes
Typing with your previously saved Typing dataset and duration. Other supported
preferences and local vocabulary edits are preserved. Saving Settings removes
the obsolete fields from the settings file.

When Typing or Translation reaches the text length, results appear immediately.
A combining accent delivered afterward can still complete the last letter and
update accuracy/WPM, with the finish time held fixed. The first other key closes
this continuation and performs its usual result-screen action. Backspace does
not reopen editing, and a result finished by the timer accepts no more input.

## Local macOS SDK override

If this Mac's default SDK is incompatible with its linker, create an ignored
`.cargo/config.toml` in the checkout and select an installed compatible SDK:

```toml
[env]
SDKROOT = { value = "/path/to/compatible/MacOSX.sdk", force = false }
```

Run `cargo run` from the checkout root. An explicitly set `SDKROOT` keeps
priority. This is a local workaround, not a required SDK version for other
machines; remove the SDKROOT entry after repairing or updating the toolchain.
Delete the config file only if it contains no other local settings.

## Adding exercise data

Every built-in set uses one JSON file in `resources/datasets/`. See
[the dataset format and required checks](docs/datasets.md). Existing SQLite
sets are preserved when a resource changes; new ids are added at startup.

## Spanish course

Start the Spanish tutor with `wpm-es` after installation, or run
`cargo run --locked --bin wpm-es` from this checkout. `cargo run --locked`
starts the original typing application.

Use Up/Down to select the module, section or mode and Left/Right to change it.
Module labels include their Spanish introduction from the preserved course
structure, such as `1. ¡Hola!`. Long labels shorten to fit the terminal.
Each change is saved immediately, including the last section visited in each
module. Enter starts the activity. Escape returns home from an activity and
exits from the home screen. Set `WPM_ES_DATA_DIR` to override the course's own
data directory.

Training uses the same typing screen as Translation. Type each full pair,
Russian text followed by a space and Spanish text. Enter types the line break
between pairs. Backspace can return to earlier pairs. The whole selected
section is typed once in catalog order without a time limit. Completion is
automatic at the last character, including when characters are incorrect.
The result shows typing accuracy, WPM and elapsed time. Repeat starts with
empty input and fresh statistics. Practice of exam mistakes types just those
pairs and preserves that subset when repeated.

Training uses Translation's default case-sensitive rules and physical layout
conversion. Spanish positions accept simplified accents, including n for ñ.
Canonical spelling is shown. The exam counts whole correct answers. Its centered input follows the cursor
through wrapping and resizing. After the last question, the result offers
"Посмотреть ошибки" to open a separate review with your answers and the correct
translations. PgUp/PgDn scroll the review; Escape returns to the result menu.
"Потренировать ошибки" starts practice directly. At 100%, the result offers
repeat and return. Exam answers ignore case and surrounding
spaces while preserving Spanish accents and ñ. Results last for the session.

The offline course covers the published [Spanish from Russian vocabulary](https://duome.eu/vocabulary/ru/es/skills).
Later topics span five consecutive sections sharing a vocabulary list. See the
[dataset guide](docs/datasets.md) for sources and validation.
