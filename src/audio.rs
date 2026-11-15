//! Cancellable eSpeak NG operations. Child output never reaches the terminal.
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioError {
    MissingProgram,
    MissingVoice,
    Process,
    Timeout,
}
impl AudioError {
    pub fn message(self) -> &'static str {
        match self {
            Self::MissingProgram => {
                "Audio unavailable: install eSpeak NG or set WPM_TTS_BIN. Esc: menu"
            }
            Self::MissingVoice => {
                "Spanish voice es unavailable: install eSpeak NG Spanish voice. Esc: menu"
            }
            Self::Process => "Audio failed. F2: retry, Esc: menu",
            Self::Timeout => "Audio timed out. F2: retry, Esc: menu",
        }
    }
}
#[derive(Debug)]
pub struct AudioEvent {
    pub operation: u64,
    pub result: Result<(), AudioError>,
}
#[derive(Debug)]
struct Job {
    cancel: Arc<AtomicBool>,
    child: Arc<Mutex<Option<std::process::Child>>>,
    worker: thread::JoinHandle<()>,
}
impl Job {
    fn stop_child(&self) {
        self.cancel.store(true, Ordering::Release);
        if let Some(child) = self.child.lock().unwrap().as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
#[derive(Debug)]
pub struct Audio {
    next: u64,
    active: Option<Job>,
    retired: Vec<Job>,
    tx: mpsc::Sender<AudioEvent>,
    rx: mpsc::Receiver<AudioEvent>,
}
impl Default for Audio {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            next: 0,
            active: None,
            retired: Vec::new(),
            tx,
            rx,
        }
    }
}
impl Audio {
    pub fn check(&mut self) -> u64 {
        self.start(None)
    }
    pub fn speak(&mut self, word: String) -> u64 {
        self.start(Some(word))
    }
    fn start(&mut self, word: Option<String>) -> u64 {
        self.cancel();
        self.next += 1;
        let operation = self.next;
        let cancel = Arc::new(AtomicBool::new(false));
        let child = Arc::new(Mutex::new(None));
        let worker_child = child.clone();
        let worker_cancel = cancel.clone();
        let tx = self.tx.clone();
        let worker = thread::spawn(move || {
            let cancel = worker_cancel;
            let binary = std::env::var_os("WPM_TTS_BIN").unwrap_or_else(|| "espeak-ng".into());
            let result = if let Some(word) = word {
                run_controlled(
                    &binary,
                    &["--stdin", "-b", "1", "-v", "es", "-s", "150"],
                    Some(&word),
                    Duration::from_secs(15),
                    &cancel,
                    &worker_child,
                )
                .map(|_| ())
            } else {
                run_controlled(
                    &binary,
                    &["--version"],
                    None,
                    Duration::from_secs(5),
                    &cancel,
                    &worker_child,
                )
                .and_then(|_| {
                    let voices = run_controlled(
                        &binary,
                        &["--voices=es"],
                        None,
                        Duration::from_secs(5),
                        &cancel,
                        &worker_child,
                    )?;
                    if voices
                        .lines()
                        .any(|line| line.split_whitespace().nth(1) == Some("es"))
                    {
                        Ok(())
                    } else {
                        Err(AudioError::MissingVoice)
                    }
                })
            };
            if !cancel.load(Ordering::Acquire) {
                let _ = tx.send(AudioEvent { operation, result });
            }
        });
        self.active = Some(Job {
            cancel,
            child,
            worker,
        });
        operation
    }
    pub fn poll(&self) -> Option<AudioEvent> {
        self.rx.try_recv().ok()
    }
    pub fn cancel(&mut self) {
        if let Some(job) = self.active.take() {
            job.cancel.store(true, Ordering::Release);
            self.retired.push(job);
        }
        let mut pending = Vec::new();
        for job in self.retired.drain(..) {
            if job.worker.is_finished() {
                let _ = job.worker.join();
            } else {
                pending.push(job);
            }
        }
        self.retired = pending;
    }
    /// UI cancellation is asynchronous; shutdown confirms child termination before process exit.
    pub fn shutdown(&mut self) {
        self.cancel();
        for job in &self.retired {
            job.stop_child();
        }
        let deadline = Instant::now() + Duration::from_secs(1);
        while self.retired.iter().any(|job| !job.worker.is_finished()) && Instant::now() < deadline
        {
            thread::sleep(Duration::from_millis(5));
        }
        for job in self.retired.drain(..) {
            // Kill again to cover a spawn racing with the first shutdown pass.
            job.stop_child();
            if job.worker.is_finished() {
                let _ = job.worker.join();
            }
        }
    }
}
impl Drop for Audio {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
fn run(
    binary: &std::ffi::OsStr,
    args: &[&str],
    input: Option<&str>,
    timeout: Duration,
    cancel: &AtomicBool,
) -> Result<String, AudioError> {
    run_controlled(
        binary,
        args,
        input,
        timeout,
        cancel,
        &Arc::new(Mutex::new(None)),
    )
}

fn run_controlled(
    binary: &std::ffi::OsStr,
    args: &[&str],
    input: Option<&str>,
    timeout: Duration,
    cancel: &AtomicBool,
    child_slot: &Arc<Mutex<Option<std::process::Child>>>,
) -> Result<String, AudioError> {
    if cancel.load(Ordering::Acquire) {
        return Err(AudioError::Process);
    }
    let mut child = Command::new(binary)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AudioError::MissingProgram
            } else {
                AudioError::Process
            }
        })?;
    // Drain both pipes concurrently so verbose or broken programs cannot deadlock us.
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdin = child.stdin.take();
    *child_slot.lock().unwrap() = Some(child);
    let (output_tx, output_rx) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = stdout;
        let mut text = Vec::new();
        let mut chunk = [0u8; 4096];
        while let Ok(count) = reader.read(&mut chunk) {
            if count == 0 {
                break;
            }
            if text.len() < 65536 {
                text.extend_from_slice(&chunk[..count.min(65536 - text.len())]);
            }
        }
        let _ = output_tx.send(text);
    });
    thread::spawn(move || {
        let mut sink = std::io::sink();
        let _ = std::io::copy(&mut { stderr }, &mut sink);
    });
    if let (Some(input), Some(mut stdin)) = (input.map(str::to_owned), stdin) {
        thread::spawn(move || {
            let _ = stdin.write_all(input.as_bytes());
        });
    }
    let started = Instant::now();
    let result = loop {
        let mut guard = child_slot.lock().unwrap();
        let child = guard.as_mut().unwrap();
        if cancel.load(Ordering::Acquire) || started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            break Err(if started.elapsed() >= timeout {
                AudioError::Timeout
            } else {
                AudioError::Process
            });
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                break if status.success() {
                    Ok(())
                } else {
                    Err(AudioError::Process)
                };
            }
            Ok(None) => {
                drop(guard);
                thread::sleep(Duration::from_millis(10));
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(AudioError::Process);
            }
        }
    };
    child_slot.lock().unwrap().take();
    // A broken wrapper may leave descendants holding its pipes. Do not wait for them.
    let text = output_rx
        .recv_timeout(Duration::from_millis(50))
        .unwrap_or_default();
    result.map(|_| String::from_utf8_lossy(&text).into_owned())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::AtomicU64;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    fn script(body: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "wpm audio contract {} {}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        path
    }
    #[test]
    fn process_contract_uses_arguments_and_utf8_stdin() {
        let path = script("[ \"$*\" = '--stdin -b 1 -v es -s 150' ] || exit 8\ncat");
        let result = run(
            path.as_os_str(),
            &["--stdin", "-b", "1", "-v", "es", "-s", "150"],
            Some("año; $(no-shell)"),
            Duration::from_secs(1),
            &AtomicBool::new(false),
        );
        assert_eq!(result, Ok("año; $(no-shell)".into()));
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn failures_and_timeout_are_bounded_even_when_descendant_holds_pipe() {
        let path = script("sleep 1 &\nwait");
        let now = Instant::now();
        assert_eq!(
            run(
                path.as_os_str(),
                &[],
                None,
                Duration::from_millis(50),
                &AtomicBool::new(false)
            ),
            Err(AudioError::Timeout)
        );
        assert!(now.elapsed() < Duration::from_millis(500));
        std::fs::remove_file(path).unwrap();
        let path = script("echo secret >&2\nexit 4");
        assert_eq!(
            run(
                path.as_os_str(),
                &[],
                None,
                Duration::from_secs(1),
                &AtomicBool::new(false)
            ),
            Err(AudioError::Process)
        );
        std::fs::remove_file(path).unwrap();
        assert_eq!(
            run(
                std::ffi::OsStr::new("/not/a/tts/program"),
                &[],
                None,
                Duration::from_secs(1),
                &AtomicBool::new(false)
            ),
            Err(AudioError::MissingProgram)
        );
    }
}
