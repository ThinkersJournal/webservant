// SPDX-License-Identifier: MIT OR Apache-2.0
//! `SIGTERM` (what `systemctl stop` and `docker stop` send) stops the server gracefully: it exits with status 0, not by the signal.
#![cfg(unix)]

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn sigterm_stops_the_server_with_a_clean_exit() {
    let dir = tempfile::tempdir().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_webservant"))
        .args([
            "serve",
            "--files",
            dir.path().to_str().unwrap(),
            "--bind",
            "127.0.0.1:0",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut out = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    out.read_line(&mut line).unwrap();
    assert!(line.starts_with("serving http://"), "{line}");

    let sent = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(sent.success());
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            // A process that did not handle SIGTERM is ended BY the signal and `success()` is false: that is what this guards.
            assert!(status.success(), "ended with {status:?}");
            return;
        }
        if start.elapsed() > Duration::from_secs(10) {
            let _ = child.kill();
            panic!("the server did not stop within 10 s of SIGTERM");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
