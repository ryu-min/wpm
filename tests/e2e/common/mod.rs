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
    pub cells: Vec<Vec<(String, termwiz::cell::CellAttributes)>>,
}

pub fn run_wpm_with_keys(keys: &[&[u8]], data_dir: &Path) -> String {
    run_wpm_capture(keys, data_dir).transcript
}

pub fn run_wpm_capture(keys: &[&[u8]], data_dir: &Path) -> TestRun {
    run_wpm_capture_with_pause(keys, data_dir, None)
}

pub fn run_wpm_capture_with_pause(
    keys: &[&[u8]],
    data_dir: &Path,
    pause_after: Option<(usize, Duration)>,
) -> TestRun {
    run_wpm_capture_with_options(
        keys,
        data_dir,
        &[],
        &pause_after.into_iter().collect::<Vec<_>>(),
    )
}

pub fn run_wpm_capture_with_options(
    keys: &[&[u8]],
    data_dir: &Path,
    environment: &[(&str, &Path)],
    pauses: &[(usize, Duration)],
) -> TestRun {
    run_wpm_capture_waiting(keys, data_dir, environment, pauses, &[])
}

pub fn run_wpm_capture_waiting(
    keys: &[&[u8]],
    data_dir: &Path,
    environment: &[(&str, &Path)],
    pauses: &[(usize, Duration)],
    expectations: &[(usize, &str)],
) -> TestRun {
    run_wpm_capture_adaptive(keys, data_dir, environment, pauses, expectations, &[])
}

pub type ScreenResponse = (usize, fn(&str) -> Vec<u8>);

/// Replace selected key chunks using the currently visible terminal screen.
/// Useful when the exercise deliberately shuffles full phrase pairs.
pub fn run_wpm_capture_adaptive(
    keys: &[&[u8]],
    data_dir: &Path,
    environment: &[(&str, &Path)],
    pauses: &[(usize, Duration)],
    expectations: &[(usize, &str)],
    responses: &[ScreenResponse],
) -> TestRun {
    run_wpm_capture_geometry(
        keys,
        data_dir,
        environment,
        pauses,
        expectations,
        responses,
        (80, 24),
    )
}

/// Keep both the real PTY and reconstructed screen at the requested geometry.
pub fn run_wpm_capture_geometry(
    keys: &[&[u8]],
    data_dir: &Path,
    environment: &[(&str, &Path)],
    pauses: &[(usize, Duration)],
    expectations: &[(usize, &str)],
    responses: &[ScreenResponse],
    geometry: (usize, usize),
) -> TestRun {
    run_binary_capture(
        keys,
        data_dir,
        environment,
        pauses,
        expectations,
        responses,
        geometry,
        false,
        false,
    )
}

pub fn run_course_capture(
    keys: &[&[u8]],
    data_dir: &Path,
    geometry: (usize, usize),
    responses: &[ScreenResponse],
) -> TestRun {
    run_binary_capture(
        keys,
        data_dir,
        &[],
        &[],
        &[],
        responses,
        geometry,
        true,
        false,
    )
}

pub fn run_course_waiting(
    keys: &[&[u8]],
    data_dir: &Path,
    geometry: (usize, usize),
    expectations: &[(usize, &str)],
) -> TestRun {
    run_binary_capture(
        keys,
        data_dir,
        &[],
        &[],
        expectations,
        &[],
        geometry,
        true,
        false,
    )
}

pub fn run_course_interrupted(keys: &[&[u8]], data_dir: &Path) -> TestRun {
    run_binary_capture(keys, data_dir, &[], &[], &[], &[], (80, 24), true, true)
}

fn run_binary_capture(
    keys: &[&[u8]],
    data_dir: &Path,
    environment: &[(&str, &Path)],
    pauses: &[(usize, Duration)],
    expectations: &[(usize, &str)],
    responses: &[ScreenResponse],
    geometry: (usize, usize),
    course: bool,
    interrupt: bool,
) -> TestRun {
    run_binary_observing(
        keys,
        data_dir,
        environment,
        pauses,
        expectations,
        responses,
        geometry,
        course,
        interrupt,
        &[],
    )
}

pub type CellObservation = (
    usize,
    fn(&str, &[(String, termwiz::cell::CellAttributes)]) -> bool,
);

pub fn run_course_observing(
    keys: &[&[u8]],
    data_dir: &Path,
    responses: &[ScreenResponse],
    observations: &[CellObservation],
) -> TestRun {
    run_binary_observing(
        keys,
        data_dir,
        &[],
        &[],
        &[],
        responses,
        (80, 24),
        true,
        false,
        observations,
    )
}

fn run_binary_observing(
    keys: &[&[u8]],
    data_dir: &Path,
    environment: &[(&str, &Path)],
    pauses: &[(usize, Duration)],
    expectations: &[(usize, &str)],
    responses: &[ScreenResponse],
    geometry: (usize, usize),
    course: bool,
    interrupt: bool,
    observations: &[CellObservation],
) -> TestRun {
    let mut command = script_command(geometry);
    if interrupt {
        command.env("WPM_PID_PATH", data_dir.join("process.pid"));
    }
    for (name, value) in environment {
        command.env(name, value);
    }
    command
        .env_remove("NO_COLOR")
        .env("TERM", "xterm-256color")
        .env(
            "WPM_BIN",
            if course {
                env!("CARGO_BIN_EXE_wpm-es")
            } else {
                env!("CARGO_BIN_EXE_wpm")
            },
        )
        .env("WPM_ES_DATA_DIR", data_dir)
        .env("WPM_DATA_DIR", data_dir)
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
    while !String::from_utf8_lossy(&captured.lock().unwrap()).contains(if course {
        if geometry.1 < 18 {
            "Увеличьте"
        } else {
            "Испанский"
        }
    } else {
        "> Quick Start"
    }) {
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
    let mut cells = Vec::with_capacity(keys.len());
    for (index, key) in keys.iter().enumerate() {
        let response = responses
            .iter()
            .find(|(at, _)| *at == index)
            .map(|(_, respond)| respond(&screen_from_output(&captured.lock().unwrap(), geometry)));
        stdin
            .write_all(response.as_deref().unwrap_or(key))
            .unwrap_or_else(|e| {
                panic!(
                    "failed to write test input: {e}\n{}",
                    String::from_utf8_lossy(&captured.lock().unwrap())
                )
            });
        stdin.flush().expect("failed to flush test input");
        thread::sleep(Duration::from_millis(75));
        for (_, duration) in pauses
            .iter()
            .filter(|(pause_index, _)| index == *pause_index)
        {
            thread::sleep(*duration);
        }
        for (_, expected) in expectations
            .iter()
            .filter(|(wait_index, _)| index == *wait_index)
        {
            let started = Instant::now();
            loop {
                let screen = screen_from_output(&captured.lock().unwrap(), geometry);
                if screen.contains(expected) {
                    break;
                }
                if started.elapsed() >= Duration::from_secs(5) {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("expected {expected:?} after key {index}\nscreen:\n{screen}");
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
        for (_, ready) in observations.iter().filter(|(at, _)| *at == index) {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                let output = captured.lock().unwrap();
                let screen = screen_from_output(&output, geometry);
                let mut surface = surface_from_output(&output, geometry);
                let cells = surface
                    .screen_cells()
                    .into_iter()
                    .flat_map(|row| {
                        row.iter()
                            .map(|c| (c.str().to_owned(), c.attrs().clone()))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                if ready(&screen, &cells) {
                    break;
                }
                assert!(
                    Instant::now() < deadline,
                    "state observation failed after key {index}: {screen}"
                );
                drop(output);
                thread::sleep(Duration::from_millis(10));
            }
        }
        let output = captured.lock().unwrap();
        screens.push(screen_from_output(&output, geometry));
        let mut surface = surface_from_output(&output, geometry);
        cells.push(
            surface
                .screen_cells()
                .into_iter()
                .flat_map(|line| {
                    line.iter()
                        .map(|cell| (cell.str().to_owned(), cell.attrs().clone()))
                        .collect::<Vec<_>>()
                })
                .collect(),
        );
    }
    if interrupt {
        let pid = fs::read_to_string(data_dir.join("process.pid")).unwrap();
        assert!(
            Command::new("kill")
                .arg("-KILL")
                .arg(pid.trim())
                .status()
                .unwrap()
                .success()
        );
    }
    drop(stdin);

    let output = wait_with_timeout(child, Duration::from_secs(5), &captured);
    reader.join().expect("failed to join stdout reader");
    let stdout = String::from_utf8_lossy(&captured.lock().unwrap()).into_owned();
    assert!(
        interrupt || output.status.success(),
        "expected successful exit\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    TestRun {
        transcript: stdout,
        screens,
        cells,
    }
}

fn screen_from_output(output: &[u8], geometry: (usize, usize)) -> String {
    surface_from_output(output, geometry).screen_chars_to_string()
}

fn surface_from_output(output: &[u8], geometry: (usize, usize)) -> Surface {
    let mut parser = Parser::new();
    let mut surface = Surface::new(geometry.0, geometry.1);
    let mut pending_text = String::new();
    for action in parser.parse_as_vec(output) {
        match &action {
            Action::Print(ch) => {
                pending_text.push(*ch);
                continue;
            }
            Action::PrintString(text) => {
                pending_text.push_str(text);
                continue;
            }
            _ => {}
        }
        if !pending_text.is_empty() {
            surface.add_change(Change::Text(std::mem::take(&mut pending_text)));
        }
        match action {
            Action::CSI(CSI::Cursor(Cursor::Position { line, col })) => {
                surface.add_change(Change::CursorPosition {
                    x: Position::Absolute(col.as_zero_based() as usize),
                    y: Position::Absolute(line.as_zero_based() as usize),
                });
            }
            Action::CSI(CSI::Sgr(sgr)) => {
                use termwiz::{cell::AttributeChange, escape::csi::Sgr};
                match sgr {
                    Sgr::Reset => {
                        surface.add_change(Change::AllAttributes(Default::default()));
                    }
                    Sgr::Foreground(color) => {
                        surface.add_change(Change::Attribute(AttributeChange::Foreground(
                            color.into(),
                        )));
                    }
                    Sgr::Underline(value) => {
                        surface.add_change(Change::Attribute(AttributeChange::Underline(value)));
                    }
                    Sgr::StrikeThrough(value) => {
                        surface
                            .add_change(Change::Attribute(AttributeChange::StrikeThrough(value)));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    if !pending_text.is_empty() {
        surface.add_change(Change::Text(pending_text));
    }
    surface
}

fn script_command(geometry: (usize, usize)) -> Command {
    let mut command = Command::new("script");
    let shell_command = format!(
        "stty rows {} cols {}; {}",
        geometry.1,
        geometry.0,
        "if [ -n \"$WPM_PID_PATH\" ]; then echo $$ > \"$WPM_PID_PATH\"; fi; exec \"$WPM_BIN\""
    );
    if cfg!(target_os = "macos") {
        command
            .arg("-q")
            .arg("/dev/null")
            .arg("sh")
            .arg("-c")
            .arg(shell_command);
    } else {
        command
            .arg("-q")
            .arg("-e")
            .arg("-c")
            .arg(shell_command)
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
        path.push(format!("wpm-e2e-{}-{nonce}-{id}", std::process::id()));
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

    /// Small shared catalog; repeated spellings with separate ids make shuffled UI
    /// exercises deterministic without changing production sampling.
    pub fn seed_spanish(&self, set_name: &str, pairs: &[(&str, &str)]) {
        let conn = rusqlite::Connection::open(self.path.join("wordset.db")).unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS vocabulary_sets (
                name TEXT PRIMARY KEY, source_lang TEXT NOT NULL,
                target_lang TEXT NOT NULL, level TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS vocabulary_entries (
                id TEXT PRIMARY KEY, set_name TEXT NOT NULL REFERENCES vocabulary_sets(name),
                ru TEXT NOT NULL, es TEXT NOT NULL
            );",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO vocabulary_sets VALUES (?1, 'ru', 'es', ?2)",
            (set_name, set_name.rsplit('_').next().unwrap()),
        )
        .unwrap();
        for (index, (ru, es)) in pairs.iter().enumerate() {
            conn.execute(
                "INSERT INTO vocabulary_entries VALUES (?1, ?2, ?3, ?4)",
                (format!("{set_name}:{index}"), set_name, ru, es),
            )
            .unwrap();
        }
    }

    pub fn write_settings(&self, overrides: serde_json::Value) {
        let mut settings = serde_json::json!({
            "quick_start_mode": "typing", "quick_start_time": 15,
            "quick_start_wordset": "en_1000", "quick_start_translation_set": "ru_en_a1",
            "translation_time": 60, "translation_set": "ru_en_a1",
            "typing_layout_conversion": false
        });
        settings
            .as_object_mut()
            .unwrap()
            .extend(overrides.as_object().unwrap().clone());
        fs::write(self.path.join("settings.json"), settings.to_string()).unwrap();
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
