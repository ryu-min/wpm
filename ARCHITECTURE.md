# Architecture

The [OpenSpec overview](openspec/specs/repository-architecture/spec.md) describes the current repository architecture. The [architecture test modules](tests/architecture/) are the primary source of truth for exact executable rules; [main.rs](tests/architecture/main.rs) is their Cargo test entry point.

The `wpm` and `wpm-es` executables each call one launcher in `src/lib.rs`.
Both share terminal initialization and restoration. The Spanish course keeps its
controller in `app/course.rs`, rendering in `ui/course.rs`, answer rules in
`domain/course.rs`, and offline loading and preferences in `storage/course.rs`.
The build assembles the course independently of the SQLite-backed typing catalog.
