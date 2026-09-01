#![cfg(unix)]

use std::{
    io::{Read, Write},
    process::Command,
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tempfile::TempDir;

type PtyChild = Box<dyn portable_pty::Child + Send + Sync>;
type PtyWriter = Box<dyn Write + Send>;

enum PtyReadEvent {
    Bytes(Vec<u8>),
    Eof,
    Error(String),
}

struct PtySession {
    child: PtyChild,
    writer: Option<PtyWriter>,
    events: Receiver<PtyReadEvent>,
    reader_thread: Option<thread::JoinHandle<()>>,
    output: Vec<u8>,
}

fn staged_repository() -> TempDir {
    let directory = tempfile::tempdir().expect("temporary repository should be created");
    for arguments in [
        ["init"].as_slice(),
        ["config", "user.email", "test@example.com"].as_slice(),
        ["config", "user.name", "Test User"].as_slice(),
    ] {
        assert!(
            Command::new("git")
                .args(arguments)
                .current_dir(directory.path())
                .status()
                .expect("git should start")
                .success()
        );
    }
    std::fs::write(directory.path().join("staged.txt"), "staged\n")
        .expect("staged file should be written");
    assert!(
        Command::new("git")
            .args(["add", "staged.txt"])
            .current_dir(directory.path())
            .status()
            .expect("git add should start")
            .success()
    );
    directory
}

fn spawn_in_pty(directory: &TempDir) -> PtySession {
    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("PTY should open");
    let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_cocommit"));
    command.cwd(directory.path());
    let child = pair
        .slave
        .spawn_command(command)
        .expect("cocommit should start in the PTY");
    drop(pair.slave);
    let reader = pair
        .master
        .try_clone_reader()
        .expect("PTY reader should open");
    let writer = pair.master.take_writer().expect("PTY writer should open");
    let (sender, events) = mpsc::channel();
    let reader_thread = thread::spawn(move || {
        loop {
            let mut buffer = [0; 1024];
            match reader.read(&mut buffer) {
                Ok(0) => {
                    let _ = sender.send(PtyReadEvent::Eof);
                    return;
                }
                Ok(bytes_read) => {
                    if sender
                        .send(PtyReadEvent::Bytes(buffer[..bytes_read].to_vec()))
                        .is_err()
                    {
                        return;
                    }
                }
                Err(error) => {
                    let _ = sender.send(PtyReadEvent::Error(error.to_string()));
                    return;
                }
            }
        }
    });

    PtySession {
        child,
        writer: Some(writer),
        events,
        reader_thread: Some(reader_thread),
        output: Vec::new(),
    }
}

impl PtySession {
    fn wait_for_terminal_ready(&mut self) {
        const ALTERNATE_SCREEN_ENTER: &[u8] = b"\x1b[?1049h";
        let deadline = Instant::now() + Duration::from_secs(5);

        loop {
            if self
                .output
                .windows(ALTERNATE_SCREEN_ENTER.len())
                .any(|window| window == ALTERNATE_SCREEN_ENTER)
            {
                return;
            }

            let remaining = deadline
                .checked_duration_since(Instant::now())
                .expect("cocommit did not initialize the terminal in time");
            match self.events.recv_timeout(remaining) {
                Ok(PtyReadEvent::Bytes(bytes)) => self.output.extend(bytes),
                Ok(PtyReadEvent::Eof) => panic!(
                    "PTY output ended before terminal initialization: {:?}",
                    String::from_utf8_lossy(&self.output)
                ),
                Ok(PtyReadEvent::Error(error)) => panic!(
                    "PTY output failed before terminal initialization: {error}; output: {:?}",
                    String::from_utf8_lossy(&self.output)
                ),
                Err(mpsc::RecvTimeoutError::Timeout) => panic!(
                    "cocommit did not initialize the terminal in time; output: {:?}",
                    String::from_utf8_lossy(&self.output)
                ),
                Err(mpsc::RecvTimeoutError::Disconnected) => panic!(
                    "PTY reader stopped before terminal initialization; output: {:?}",
                    String::from_utf8_lossy(&self.output)
                ),
            }
        }
    }

    fn write_input(&mut self, input: &[u8]) {
        let writer = self.writer.as_mut().expect("PTY writer should remain open");
        writer.write_all(input).expect("input should be sent");
        writer.flush().expect("input should be flushed");
    }

    fn process_id(&self) -> u32 {
        self.child
            .process_id()
            .expect("child should expose a process ID")
    }

    fn wait_for_exit(&mut self) -> portable_pty::ExitStatus {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = self
                .child
                .try_wait()
                .expect("child status should be readable")
            {
                return status;
            }
            assert!(Instant::now() < deadline, "cocommit did not exit in time");
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn finish_output(&mut self) -> String {
        self.writer.take();
        let deadline = Instant::now() + Duration::from_secs(5);

        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .expect("PTY output did not close in time");
            match self.events.recv_timeout(remaining) {
                Ok(PtyReadEvent::Bytes(bytes)) => self.output.extend(bytes),
                Ok(PtyReadEvent::Eof) => break,
                Ok(PtyReadEvent::Error(error)) => panic!("PTY output failed: {error}"),
                Err(mpsc::RecvTimeoutError::Timeout) => panic!("PTY output did not close in time"),
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        self.reader_thread
            .take()
            .expect("PTY reader thread should remain available")
            .join()
            .expect("PTY reader thread should not panic");
        String::from_utf8(self.output.clone()).expect("PTY output should be UTF-8")
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        self.writer.take();
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

#[test]
fn pty_cancel_exits_cleanly() {
    let directory = staged_repository();
    let mut session = spawn_in_pty(&directory);
    session.wait_for_terminal_ready();
    session.write_input(b"\x1b");

    assert!(session.wait_for_exit().success());
    let _ = session.finish_output();
}

#[test]
fn pty_space_does_not_restart_the_terminal_session() {
    let directory = staged_repository();
    let mut session = spawn_in_pty(&directory);
    session.wait_for_terminal_ready();
    session.write_input(b" \x1b");

    assert!(session.wait_for_exit().success());
    let output = session.finish_output();
    assert_eq!(
        output.matches("\x1b[?1049h").count(),
        1,
        "space should not recreate the alternate-screen session"
    );
}

#[test]
fn pty_sigterm_restores_before_terminating() {
    let directory = staged_repository();
    let mut session = spawn_in_pty(&directory);
    session.wait_for_terminal_ready();
    let process_id = session.process_id();
    assert!(
        Command::new("kill")
            .args(["-TERM", &process_id.to_string()])
            .status()
            .expect("kill should start")
            .success()
    );

    assert_eq!(session.wait_for_exit().exit_code(), 128 + 15);
    let output = session.finish_output();
    assert!(
        output.contains("\x1b[?1049h"),
        "alternate screen was entered"
    );
    assert!(
        output.contains("\x1b[?1049l"),
        "alternate screen was restored"
    );
    assert!(
        output.contains("\x1b[?2004h"),
        "bracketed paste was enabled"
    );
    assert!(
        output.contains("\x1b[?2004l"),
        "bracketed paste was disabled"
    );
    assert!(output.contains("\x1b[?25l"), "cursor was hidden");
    assert!(output.contains("\x1b[?25h"), "cursor was shown");
}
