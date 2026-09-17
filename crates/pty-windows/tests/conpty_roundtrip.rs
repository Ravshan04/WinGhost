#![cfg(windows)]

use pty_windows::{Session, ShellProfile};
use std::thread;
use std::time::{Duration, Instant};

const MARKER: &str = "__WINGHOST_CONPTY_ROUNDTRIP__";

#[test]
fn command_prompt_completes_input_output_roundtrip() {
    let session = Session::spawn_profile(ShellProfile::CommandPrompt, 80, 24)
        .expect("Command Prompt should start in ConPTY");
    session
        .send(format!("echo {MARKER}\r\n").into_bytes())
        .expect("test command should reach ConPTY");

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut output = Vec::new();
    while Instant::now() < deadline {
        for chunk in session.drain_output() {
            output.extend(chunk);
        }
        if String::from_utf8_lossy(&output).contains(MARKER) {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }

    panic!("ConPTY did not return shell output after receiving keyboard input");
}
