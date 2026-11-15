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
`ru_es_a1` through `ru_es_c2` are shared with `Dictation`. These are application
vocabulary levels, not a certified CEFR course. Translation types both sides of
each pair; Enter types the line break between pairs. Backspace can cross it.
Translation needs no audio software.

Spanish works with ENG and RU keyboard layouts. The same physical RU keys are
accepted for ENG letters (`рщдф` → `hola`; this is not transliteration). Spanish
comparison accepts `n/ñ`, `a/á`, `e/é`, `i/í`, `o/ó`, and `u/ú/ü`, preserving
case. For example, `ano` is accepted for `año` and `si` for `sí`; this convenience
does not test knowledge of accents. Canonical spelling is always shown in the
exercise or revealed answer. Exact Unicode input and composed accents work too.
The ordinary Typing layout-conversion setting remains independent.

In `Dictation`, listen and type a lowercase word. Enter checks the complete
answer after successful playback; another Enter continues. F2 repeats the audio,
Backspace edits the answer, Escape cancels to the menu, and Ctrl+C quits. The
answer and translation stay hidden until submission. One wrong submitted word
counts as one error; editing before submission and audio failures do not add
errors. Leading/trailing spaces are ignored, internal spaces are not. Choose
10, 20, or 50 distinct words (20 by default); smaller sets use all their words.
Results show correct words and errors rather than WPM. Enter restarts with a
fresh sample and zero counters. Direct and Quick Start settings are separate;
Enter saves Settings, Escape discards edits.

## Dictation audio setup

Install [eSpeak NG](https://github.com/espeak-ng/espeak-ng) separately. `wpm`
does not download a synthesizer or voices. Audio works offline with the Spanish
Spain voice `es` at 150 words/minute. Speech is synthetic; check that you can
hear and understand it on your output device before using dictation.

- macOS: `brew install espeak-ng` ([Homebrew formula](https://formulae.brew.sh/formula/espeak-ng)).
- Linux: install your distribution's `espeak-ng` package, for example
  `sudo apt install espeak-ng` on Debian/Ubuntu.
- Windows: install the official MSI from the [eSpeak NG releases](https://github.com/espeak-ng/espeak-ng/releases)
  and add its installation directory to `PATH` (see the [official guide](https://github.com/espeak-ng/espeak-ng/blob/master/docs/guide.md)).

Check the dependency and voice, then actually listen:

```sh
espeak-ng --version
espeak-ng --voices=es
espeak-ng -v es "año"
```

If the executable is outside `PATH`, set `WPM_TTS_BIN` to its full filename:
`export WPM_TTS_BIN="/path with spaces/espeak-ng"` on macOS/Linux, or
`$env:WPM_TTS_BIN = 'C:\Program Files\eSpeak NG\espeak-ng.exe'` in PowerShell.
This is a path to one executable, not a shell command or argument list.

Dictation checks the executable and the exact `es` voice before starting.
If playback fails, your input is retained; retry F2 or return with Escape.
A successful process cannot prove that speakers are connected or unmuted.
For output-device troubleshooting follow the official guide. Other modes
remain usable without the dependency.

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
