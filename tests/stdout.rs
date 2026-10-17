use std::fs::OpenOptions;
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use rustix::pty::{OpenptFlags, grantpt, openpt, ptsname, unlockpt};

/// `msg | cat` from a terminal: stdin is a terminal and stdout is a pipe. With
/// no terminal on stdin the view fails to start before writing anything, so
/// this needs a real one to show that stdout stays empty.
#[test]
fn piped_from_a_terminal_it_writes_nothing_to_stdout_and_fails() {
    let controller = openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY).expect("opening a pseudo-terminal");
    grantpt(&controller).expect("granting the pseudo-terminal");
    unlockpt(&controller).expect("unlocking the pseudo-terminal");
    let path = ptsname(&controller, Vec::new()).expect("naming the pseudo-terminal");
    let terminal = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path.to_str().expect("a UTF-8 terminal path"))
        .expect("opening the terminal side");

    let mut child = Command::new(env!("CARGO_BIN_EXE_msg"))
        .stdin(Stdio::from(terminal))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("running msg");

    // Without the guard the view starts and waits on a key, so it never exits.
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().expect("waiting on msg") {
            break Some(status);
        }
        if Instant::now() > deadline {
            child.kill().expect("stopping msg");
            child.wait().expect("reaping msg");
            break None;
        }
        thread::sleep(Duration::from_millis(20));
    };
    let mut stdout = Vec::new();
    child
        .stdout
        .take()
        .expect("piped stdout")
        .read_to_end(&mut stdout)
        .expect("reading stdout");

    assert_eq!(status.and_then(|status| status.code()), Some(1));
    assert!(stdout.is_empty(), "{} bytes on stdout", stdout.len());
}
