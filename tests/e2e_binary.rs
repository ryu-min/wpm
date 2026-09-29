#![cfg(unix)]

use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[test]
fn binary_starts_on_menu_and_exits_on_escape() {
    let data_dir = TestDataDir::new();

    let output = run_wmp_with_input(b"\x1b", data_dir.path());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "expected successful exit\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        stderr
    );
    assert!(
        stdout.contains("Quick Start"),
        "expected menu to contain Quick Start\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("Settings"),
        "expected menu to contain Settings\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
}

fn run_wmp_with_input(input: &[u8], data_dir: &Path) -> Output {
    let mut command = script_command();
    command
        .env("TERM", "xterm-256color")
        .env("WMP_BIN", env!("CARGO_BIN_EXE_wmp"))
        .env("WMP_DATA_DIR", data_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().expect("failed to spawn app under script");
    let mut stdin = child.stdin.take().expect("failed to open script stdin");
    stdin.write_all(input).expect("failed to write test input");
    drop(stdin);

    wait_with_timeout(child, Duration::from_secs(5))
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

fn wait_with_timeout(mut child: std::process::Child, timeout: Duration) -> Output {
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
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        thread::sleep(Duration::from_millis(20));
    }
}

struct TestDataDir {
    path: PathBuf,
}

impl TestDataDir {
    fn new() -> Self {
        let mut path = env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is before unix epoch")
            .as_nanos();
        path.push(format!("wmp-e2e-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).expect("failed to create temporary data dir");

        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDataDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
