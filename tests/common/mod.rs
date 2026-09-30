#![allow(dead_code)]

use std::{
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use termwiz::{
    escape::{Action, CSI, csi::Cursor, parser::Parser},
    surface::{Change, Position, Surface},
};

pub const QUICK_START: &str = "Quick Start";
pub const TRANSLATION: &str = "Translation";
pub const SELECT_MODE: &str = "Select Mode";
pub const SETTINGS: &str = "Settings";
pub const EXIT: &str = "Exit";

pub const MENU_LABELS: [&str; 5] = [QUICK_START, TRANSLATION, SELECT_MODE, SETTINGS, EXIT];
pub const TEST_WORD: &str = "abc";
pub const TEST_SOURCE: &str = "кот";
pub const TEST_TRANSLATION: &str = "cat";
pub const TEST_TRANSLATION_TEXT: &str = "кот cat";
pub const PERFECT_ACCURACY: &str = "Accuracy: 100%";

pub const UP: &[u8] = b"\x1b[A";
pub const DOWN: &[u8] = b"\x1b[B";
pub const RIGHT: &[u8] = b"\x1b[C";
pub const LEFT: &[u8] = b"\x1b[D";
pub const ENTER: &[u8] = b"\r";
pub const ESC: &[u8] = b"\x1b";
pub const CTRL_C: &[u8] = b"\x03";

pub struct TestRun {
    pub transcript: String,
    pub screens: Vec<String>,
}

pub fn run_wmp_with_keys(keys: &[&[u8]], data_dir: &Path) -> String {
    run_wmp_capture(keys, data_dir).transcript
}

pub fn run_wmp_capture(keys: &[&[u8]], data_dir: &Path) -> TestRun {
    run_wmp_capture_with_pause(keys, data_dir, None)
}

pub fn run_wmp_capture_with_pause(
    keys: &[&[u8]],
    data_dir: &Path,
    pause_after: Option<(usize, Duration)>,
) -> TestRun {
    let mut command = script_command();
    command
        .env("TERM", "xterm-256color")
        .env("WMP_BIN", env!("CARGO_BIN_EXE_wmp"))
        .env("WMP_DATA_DIR", data_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().expect("failed to spawn app under script");
    let mut stdout_pipe = child.stdout.take().expect("failed to open script stdout");
    let captured = Arc::new(Mutex::new(Vec::new()));
    let reader_captured = Arc::clone(&captured);
    let reader = thread::spawn(move || {
        let mut chunk = [0; 4096];
        loop {
            let count = stdout_pipe
                .read(&mut chunk)
                .expect("failed to read script stdout");
            if count == 0 {
                break;
            }
            reader_captured
                .lock()
                .unwrap()
                .extend_from_slice(&chunk[..count]);
        }
    });

    let started = Instant::now();
    while !String::from_utf8_lossy(&captured.lock().unwrap()).contains("> Quick Start") {
        if started.elapsed() >= Duration::from_secs(5) {
            let _ = child.kill();
            let _ = child.wait();
            reader.join().expect("failed to join stdout reader");
            panic!(
                "app did not show initial menu\nstdout:\n{}",
                String::from_utf8_lossy(&captured.lock().unwrap())
            );
        }
        thread::sleep(Duration::from_millis(10));
    }

    let mut stdin = child.stdin.take().expect("failed to open script stdin");
    let mut screens = Vec::with_capacity(keys.len());
    for (index, key) in keys.iter().enumerate() {
        stdin.write_all(key).expect("failed to write test input");
        stdin.flush().expect("failed to flush test input");
        thread::sleep(Duration::from_millis(75));
        if let Some((pause_index, duration)) = pause_after
            && index == pause_index
        {
            thread::sleep(duration);
        }
        screens.push(screen_from_output(&captured.lock().unwrap()));
    }
    drop(stdin);

    let output = wait_with_timeout(child, Duration::from_secs(5), &captured);
    reader.join().expect("failed to join stdout reader");
    let stdout = String::from_utf8_lossy(&captured.lock().unwrap()).into_owned();
    assert!(
        output.status.success(),
        "expected successful exit\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    TestRun {
        transcript: stdout,
        screens,
    }
}

fn screen_from_output(output: &[u8]) -> String {
    let mut parser = Parser::new();
    let mut surface = Surface::new(80, 24);
    for action in parser.parse_as_vec(output) {
        match action {
            Action::Print(ch) => {
                surface.add_change(Change::Text(ch.to_string()));
            }
            Action::PrintString(text) => {
                surface.add_change(Change::Text(text));
            }
            Action::CSI(CSI::Cursor(Cursor::Position { line, col })) => {
                surface.add_change(Change::CursorPosition {
                    x: Position::Absolute(col.as_zero_based() as usize),
                    y: Position::Absolute(line.as_zero_based() as usize),
                });
            }
            _ => {}
        }
    }
    surface.screen_chars_to_string()
}

fn script_command() -> Command {
    let mut command = Command::new("script");

    if cfg!(target_os = "macos") {
        command
            .arg("-q")
            .arg("/dev/null")
            .arg("sh")
            .arg("-c")
            .arg("stty rows 24 cols 80; exec \"$WMP_BIN\"");
    } else {
        command
            .arg("-q")
            .arg("-e")
            .arg("-c")
            .arg("stty rows 24 cols 80; exec \"$WMP_BIN\"")
            .arg("/dev/null");
    }

    command
}

fn wait_with_timeout(
    mut child: std::process::Child,
    timeout: Duration,
    captured: &Arc<Mutex<Vec<u8>>>,
) -> std::process::Output {
    let started = Instant::now();

    loop {
        if child.try_wait().expect("failed to poll child").is_some() {
            return child
                .wait_with_output()
                .expect("failed to collect child output");
        }

        if started.elapsed() >= timeout {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .expect("failed to collect timed out child output");
            panic!(
                "timed out waiting for app to exit\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&captured.lock().unwrap()),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        thread::sleep(Duration::from_millis(20));
    }
}

pub struct TestDataDir {
    path: PathBuf,
}

static NEXT_DIR_ID: AtomicU64 = AtomicU64::new(0);

impl TestDataDir {
    pub fn new() -> Self {
        let mut path = env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is before unix epoch")
            .as_nanos();
        let id = NEXT_DIR_ID.fetch_add(1, Ordering::Relaxed);
        path.push(format!("wmp-e2e-{}-{nonce}-{id}", std::process::id()));
        fs::create_dir(&path).expect("failed to create temporary data dir");

        Self { path }
    }

    pub fn seed_wordset(&self, word: &str) {
        let conn = rusqlite::Connection::open(self.path.join("wordset.db"))
            .expect("failed to open test database");
        conn.execute_batch(
            "CREATE TABLE word_sets (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                language TEXT NOT NULL,
                word_count INTEGER NOT NULL,
                words TEXT NOT NULL
            );",
        )
        .expect("failed to create word_sets table");
        conn.execute(
            "INSERT INTO word_sets (name, language, word_count, words) VALUES ('en_1000', 'en', 1, ?1)",
            [word],
        )
        .expect("failed to seed wordset");
    }

    pub fn seed_translation(&self, source: &str, target: &str) {
        let conn = rusqlite::Connection::open(self.path.join("wordset.db"))
            .expect("failed to open test database");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS translation_sets (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                source_lang TEXT NOT NULL,
                target_lang TEXT NOT NULL,
                pair_count INTEGER NOT NULL,
                pairs_json TEXT NOT NULL
            );",
        )
        .expect("failed to create translation_sets table");
        let pairs = serde_json::json!([{ "ru": source, "en": target }]).to_string();
        conn.execute(
            "INSERT INTO translation_sets (name, source_lang, target_lang, pair_count, pairs_json)
             VALUES ('ru_en_a1', 'ru', 'en', 1, ?1)",
            [pairs],
        )
        .expect("failed to seed translation set");
    }
}

impl AsRef<Path> for TestDataDir {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDataDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
