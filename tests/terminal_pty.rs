#![cfg(unix)]

use std::{
    io::{Read, Write},
    process::Command,
    thread,
    time::{Duration, Instant},
};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tempfile::TempDir;

type PtyChild = Box<dyn portable_pty::Child + Send + Sync>;
type PtyReader = Box<dyn Read + Send>;
type PtyWriter = Box<dyn Write + Send>;

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

fn spawn_in_pty(directory: &TempDir) -> (PtyChild, PtyReader, PtyWriter) {
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
    (child, reader, writer)
}

fn wait_for_exit(child: &mut dyn portable_pty::Child) -> portable_pty::ExitStatus {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().expect("child status should be readable") {
            return status;
        }
        assert!(Instant::now() < deadline, "cocommit did not exit in time");
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn pty_cancel_exits_cleanly() {
    let directory = staged_repository();
    let (mut child, _reader, mut writer) = spawn_in_pty(&directory);
    writer.write_all(b"\x1b").expect("escape should be sent");
    writer.flush().expect("escape should be flushed");

    assert!(wait_for_exit(child.as_mut()).success());
}

#[test]
fn pty_sigterm_restores_before_terminating() {
    let directory = staged_repository();
    let (mut child, mut reader, writer) = spawn_in_pty(&directory);
    thread::sleep(Duration::from_millis(200));
    let process_id = child
        .process_id()
        .expect("child should expose a process ID");
    assert!(
        Command::new("kill")
            .args(["-TERM", &process_id.to_string()])
            .status()
            .expect("kill should start")
            .success()
    );

    assert_eq!(wait_for_exit(child.as_mut()).exit_code(), 128 + 15);
    drop(writer);

    let mut output = String::new();
    reader
        .read_to_string(&mut output)
        .expect("PTY output should be readable");
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
