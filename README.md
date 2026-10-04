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
