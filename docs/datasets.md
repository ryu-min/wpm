# Adding a built-in exercise dataset

All built-in datasets use the same descriptor format below. Add one UTF-8 JSON file at `resources/datasets/<id>.json`. The build discovers
files directly in that directory, validates them, and embeds them. No Rust,
manifest, registry, or individual dataset test needs editing. Repository changes
still require the usual OpenSpec planning and implementation workflow.

```json
{
  "schema_version": 1,
  "id": "cafe_practice",
  "description": "Кафе и знакомство",
  "profile": "spanish-phrases",
  "level": "beginner",
  "entries": [
    {"id": "cafe_practice:greeting", "ru": "привет", "target": "Hola"}
  ]
}
```

The id must match the filename and `[a-z][a-z0-9_]*`; no language prefix is
required. It must be unique across the discovered descriptors.
Description is required; level is optional (defaults to `custom`), and if present
must be a nonempty string. Unknown fields, schema versions, and profiles fail
the build. Errors name the source file and offending entry where applicable.

| Profile | Modes | Entries example | Policy |
| --- | --- | --- | --- |
| `typing-ru` | Typing | `["кот", "дом"]` | Existing Typing RU/ENG layout setting |
| `typing-en` | Typing | `["cat", "house"]` | Existing Typing RU/ENG layout setting |
| `translation-ru-en` | Translation | `[{"id":"example:cat","ru":"кот","target":"cat"}]` | Existing case and physical keyboard rules |
| `spanish-words` | Translation, Dictation | `[{"id":"example:coffee","ru":"кофе","target":"café"}]` | Lowercase single-word Spanish; strict case |
| `spanish-phrases` | Translation, Dictation | `[{"id":"example:name","ru":"Меня зовут Серхио","target":"Me llamo Sergio"}]` | Dictation ignores case; Translation checks case |

Replace the example entry id prefix with your dataset id. Typing entries are
nonempty strings without whitespace or controls. Translation entries have
exactly `id`, `ru`, and `target`; ids must start with `<dataset id>:` and have a
nonempty unique suffix. Text must be NFC, nonempty, without leading/trailing
whitespace or control characters. Russian source text must contain Russian
letters. Targets are unique within a dataset by exact canonical NFC spelling;
`si`, `sí`, `Hola`, and `hola` remain distinct. Overlap between datasets is fine.
Spanish targets contain Spanish letters; the phrases profile also permits
single internal spaces. Punctuation and arbitrary sentence formats are outside
v1. Profiles are closed: new languages, modes, or semantics require a code change.

Both Spanish profiles retain optional accents and RU physical keys, canonical
TTS/revealed answers, hidden Dictation targets before submission, and one task
per entry, including a complete phrase. They do not accept omitted words.
The structural validator cannot verify a translation's meaning: edit and review
the material before shipping it.

From the checkout root, run the required checks:

```sh
cargo test --locked --all-features --all-targets
python3 scripts/check_dataset_build.py
python3 scripts/validate_spanish_catalog.py
cargo package --list --allow-dirty
```

The second command is required Unix acceptance, separate from Cargo tests. It
copies the project to a temporary directory, builds once, adds only a JSON file,
rebuilds with the same target directory, and launches the standalone binary
from an empty directory through a PTY. It checks add/change/delete detection
and invalid data/collision diagnostics. It needs Python 3, Cargo, and `script`.
On a Mac requiring a local SDK override, the temporary copy retains the local
`.cargo/config.toml`; that file remains excluded from the package. Unix PTY
results do not establish Windows coverage. The six Spanish A1–C2 sets use the same descriptors and general format checks;
the Spanish catalog validator adds their existing content constraints.

After the checks, rebuild with `cargo build --locked` or install the checkout
with `cargo install --path . --locked`. The binary needs no source directory,
network download, or runtime dataset files. Choose the new id in the existing
Settings or Select Mode for its supported modes.

A new id is added to SQLite at startup even when other datasets already exist.
Existing rows, edits, and settings are preserved: changing or deleting a built-in
file does **not** update or delete an already stored dataset. Inspect changed
built-ins with a fresh isolated `WPM_DATA_DIR`; do not delete a user's database
to deliver a vocabulary update. There is no runtime import or editor in v1.

All 19 original sets use this directory and the same loader. The old TXT and
array-only JSON resources and their registry have been removed. Migration keeps
the original sequences and text, including 4977 words in `ru_5000`; RU→EN
entry ids preserve the old zero-based `<set id>:<index>` identity. Existing
local SQLite sets without a descriptor still use their stored type/languages.
