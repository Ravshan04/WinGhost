//! Safe application boundary around the native Windows pseudoconsole.

use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::fmt;
use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// Errors produced while starting or communicating with a terminal session.
#[derive(Debug)]
pub struct SessionError(String);

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for SessionError {}

/// A running shell connected to a native pseudoconsole.
pub struct Session {
    master: Box<dyn MasterPty + Send>,
    input: Sender<Vec<u8>>,
    output: Receiver<Vec<u8>>,
    child: Box<dyn Child + Send + Sync>,
}

impl Session {
    /// Starts the preferred Windows `PowerShell` executable in a pseudoconsole.
    ///
    /// # Errors
    ///
    /// Returns an error when the pseudoconsole, shell process, I/O handles, or
    /// worker threads cannot be created.
    pub fn spawn_default(columns: u16, rows: u16) -> Result<Self, SessionError> {
        let shell = preferred_shell();
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(pty_size(columns, rows, 0, 0))
            .map_err(|error| SessionError(format!("Could not create ConPTY: {error}")))?;

        let mut command = CommandBuilder::new(&shell);
        command.arg("-NoLogo");
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        if let Ok(directory) = std::env::current_dir() {
            command.cwd(directory);
        }

        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|error| SessionError(format!("Could not start {shell}: {error}")))?;
        drop(pair.slave);

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| SessionError(format!("Could not open ConPTY output: {error}")))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|error| SessionError(format!("Could not open ConPTY input: {error}")))?;

        let (output_sender, output) = mpsc::channel();
        thread::Builder::new()
            .name("winghost-pty-reader".to_owned())
            .spawn(move || read_output(reader, &output_sender))
            .map_err(|error| SessionError(format!("Could not start output reader: {error}")))?;

        let (input, input_receiver) = mpsc::channel();
        thread::Builder::new()
            .name("winghost-pty-writer".to_owned())
            .spawn(move || write_input(writer, input_receiver))
            .map_err(|error| SessionError(format!("Could not start input writer: {error}")))?;

        Ok(Self {
            master: pair.master,
            input,
            output,
            child,
        })
    }

    /// Queues keyboard or paste input for the shell.
    ///
    /// # Errors
    ///
    /// Returns an error after the terminal input worker has stopped.
    pub fn send(&self, bytes: impl Into<Vec<u8>>) -> Result<(), SessionError> {
        self.input
            .send(bytes.into())
            .map_err(|_| SessionError("The terminal input stream has closed".to_owned()))
    }

    /// Drains currently available output without blocking the UI thread.
    #[must_use]
    pub fn drain_output(&self) -> Vec<Vec<u8>> {
        let mut chunks = Vec::new();
        while let Ok(chunk) = self.output.try_recv() {
            chunks.push(chunk);
        }
        chunks
    }

    /// Resizes the pseudoconsole and notifies the child process.
    ///
    /// # Errors
    ///
    /// Returns an error when the native pseudoconsole rejects the new size.
    pub fn resize(
        &self,
        columns: u16,
        rows: u16,
        pixel_width: u16,
        pixel_height: u16,
    ) -> Result<(), SessionError> {
        self.master
            .resize(pty_size(columns, rows, pixel_width, pixel_height))
            .map_err(|error| SessionError(format!("Could not resize ConPTY: {error}")))
    }

    /// Returns whether the child process is still running.
    pub fn is_running(&mut self) -> bool {
        self.child.try_wait().is_ok_and(|status| status.is_none())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

fn pty_size(columns: u16, rows: u16, pixel_width: u16, pixel_height: u16) -> PtySize {
    PtySize {
        rows,
        cols: columns,
        pixel_width,
        pixel_height,
    }
}

fn preferred_shell() -> String {
    let modern = std::path::Path::new(r"C:\Program Files\PowerShell\7\pwsh.exe");
    if modern.is_file() {
        modern.to_string_lossy().into_owned()
    } else {
        "powershell.exe".to_owned()
    }
}

fn read_output(mut reader: Box<dyn Read + Send>, sender: &Sender<Vec<u8>>) {
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(count) if sender.send(buffer[..count].to_vec()).is_err() => break,
            Ok(_) => {}
        }
    }
}

fn write_input(mut writer: Box<dyn Write + Send>, receiver: Receiver<Vec<u8>>) {
    for bytes in receiver {
        if writer.write_all(&bytes).is_err() || writer.flush().is_err() {
            break;
        }
    }
}
