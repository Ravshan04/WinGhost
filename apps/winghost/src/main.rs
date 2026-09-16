#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{
    self, Color32, Event, FontFamily, FontId, Key, RichText, Stroke, TextFormat, text::LayoutJob,
};
use pty_windows::Session;
use renderer::{Rgb, Theme};
use std::time::Duration;
use terminal_core::{ScreenSnapshot, Terminal};

const INITIAL_COLUMNS: u16 = 100;
const INITIAL_ROWS: u16 = 30;
const FONT_SIZE: f32 = 15.0;
const CELL_WIDTH: f32 = 8.7;
const CELL_HEIGHT: f32 = 19.0;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WinGhost")
            .with_inner_size([980.0, 650.0])
            .with_min_inner_size([560.0, 360.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WinGhost",
        options,
        Box::new(|creation_context| Ok(Box::new(WinGhostApp::new(creation_context)))),
    )
}

struct WinGhostApp {
    terminal: Terminal,
    session: Option<Session>,
    theme: Theme,
    status: String,
    size: (u16, u16),
}

impl WinGhostApp {
    fn new(context: &eframe::CreationContext<'_>) -> Self {
        context.egui_ctx.set_visuals(egui::Visuals::dark());
        let mut app = Self {
            terminal: Terminal::new(INITIAL_COLUMNS, INITIAL_ROWS),
            session: None,
            theme: Theme::default(),
            status: String::new(),
            size: (INITIAL_COLUMNS, INITIAL_ROWS),
        };
        app.start_session();
        app
    }

    fn start_session(&mut self) {
        self.terminal = Terminal::new(self.size.0, self.size.1);
        match Session::spawn_default(self.size.0, self.size.1) {
            Ok(session) => {
                self.session = Some(session);
                "PowerShell • ConPTY".clone_into(&mut self.status);
            }
            Err(error) => {
                self.session = None;
                self.status = format!("Could not start terminal: {error}");
            }
        }
    }

    fn receive_output(&mut self) {
        if let Some(session) = &self.session {
            for chunk in session.drain_output() {
                self.terminal.process(&chunk);
            }
        }
    }

    fn send(&mut self, bytes: impl Into<Vec<u8>>) {
        if let Some(session) = &self.session
            && let Err(error) = session.send(bytes)
        {
            self.status = error.to_string();
        }
    }

    fn handle_input(&mut self, context: &egui::Context) {
        let events = context.input(|input| input.events.clone());
        for event in events {
            match event {
                Event::Text(text) => self.send(text.into_bytes()),
                Event::Paste(text) => self.send(text.replace('\n', "\r\n").into_bytes()),
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    if let Some(bytes) = key_bytes(key, modifiers) {
                        self.send(bytes);
                    }
                }
                _ => {}
            }
        }
    }

    fn resize(&mut self, available: egui::Vec2) {
        let columns = bounded_cell_count(available.x, CELL_WIDTH, 20, 300);
        let rows = bounded_cell_count(available.y, CELL_HEIGHT, 5, 150);
        if self.size == (columns, rows) {
            return;
        }

        self.size = (columns, rows);
        self.terminal.resize(columns, rows);
        if let Some(session) = &self.session {
            let pixel_width = columns.saturating_mul(9);
            let pixel_height = rows.saturating_mul(19);
            if let Err(error) = session.resize(columns, rows, pixel_width, pixel_height) {
                self.status = error.to_string();
            }
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn bounded_cell_count(available: f32, cell_size: f32, minimum: u16, maximum: u16) -> u16 {
    // Clamp before casting so the finite value is always within the u16 range.
    (available / cell_size)
        .floor()
        .clamp(f32::from(minimum), f32::from(maximum)) as u16
}

impl eframe::App for WinGhostApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.receive_output();
        self.handle_input(context);
        context.request_repaint_after(Duration::from_millis(16));

        egui::TopBottomPanel::top("toolbar").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("WinGhost")
                        .strong()
                        .color(Color32::from_rgb(110, 168, 254)),
                );
                ui.separator();
                ui.label(&self.status);
                ui.separator();
                ui.label(format!("{} × {}", self.size.0, self.size.1));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("New session").clicked() {
                        self.start_session();
                    }
                    if ui.button("Copy screen").clicked() {
                        context.copy_text(self.terminal.contents());
                    }
                });
            });
        });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(rgb(self.theme.background))
                    .inner_margin(8.0),
            )
            .show(context, |ui| {
                self.resize(ui.available_size());
                let snapshot = self.terminal.snapshot();
                let job = terminal_layout(&snapshot, self.theme);
                ui.add(egui::Label::new(job).selectable(false));
            });
    }
}

fn terminal_layout(snapshot: &ScreenSnapshot, theme: Theme) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;
    job.break_on_newline = true;

    for row in 0..snapshot.rows {
        for column in 0..snapshot.columns {
            let Some(cell) = snapshot.cell(row, column) else {
                continue;
            };
            if cell.wide_continuation {
                continue;
            }

            let cursor = !snapshot.cursor_hidden && snapshot.cursor == (row, column);
            let mut foreground = theme.resolve(cell.foreground, false);
            let mut background = theme.resolve(cell.background, true);
            if cell.style.inverse() || cursor {
                std::mem::swap(&mut foreground, &mut background);
            }
            if cursor && cell.contents.is_empty() {
                background = theme.cursor;
            }

            let contents = if cell.contents.is_empty() {
                " "
            } else {
                &cell.contents
            };
            job.append(
                contents,
                0.0,
                TextFormat {
                    font_id: FontId::new(FONT_SIZE, FontFamily::Monospace),
                    color: rgb(foreground),
                    background: rgb(background),
                    italics: cell.style.italic(),
                    underline: if cell.style.underline() {
                        Stroke::new(1.0, rgb(foreground))
                    } else {
                        Stroke::NONE
                    },
                    line_height: Some(CELL_HEIGHT),
                    ..Default::default()
                },
            );
        }
        if row + 1 < snapshot.rows {
            job.append(
                "\n",
                0.0,
                TextFormat {
                    font_id: FontId::new(FONT_SIZE, FontFamily::Monospace),
                    color: rgb(theme.foreground),
                    line_height: Some(CELL_HEIGHT),
                    ..Default::default()
                },
            );
        }
    }
    job
}

fn key_bytes(key: Key, modifiers: egui::Modifiers) -> Option<Vec<u8>> {
    if modifiers.ctrl && !modifiers.shift && !modifiers.alt {
        let letter = match key {
            Key::A => b'a',
            Key::B => b'b',
            Key::C => b'c',
            Key::D => b'd',
            Key::E => b'e',
            Key::F => b'f',
            Key::G => b'g',
            Key::H => b'h',
            Key::I => b'i',
            Key::J => b'j',
            Key::K => b'k',
            Key::L => b'l',
            Key::M => b'm',
            Key::N => b'n',
            Key::O => b'o',
            Key::P => b'p',
            Key::Q => b'q',
            Key::R => b'r',
            Key::S => b's',
            Key::T => b't',
            Key::U => b'u',
            Key::V => b'v',
            Key::W => b'w',
            Key::X => b'x',
            Key::Y => b'y',
            Key::Z => b'z',
            _ => return None,
        };
        return Some(vec![letter & 0x1f]);
    }

    let sequence: &[u8] = match key {
        Key::Enter => b"\r",
        Key::Backspace => b"\x7f",
        Key::Tab => b"\t",
        Key::Escape => b"\x1b",
        Key::ArrowUp => b"\x1b[A",
        Key::ArrowDown => b"\x1b[B",
        Key::ArrowRight => b"\x1b[C",
        Key::ArrowLeft => b"\x1b[D",
        Key::Home => b"\x1b[H",
        Key::End => b"\x1b[F",
        Key::Insert => b"\x1b[2~",
        Key::Delete => b"\x1b[3~",
        Key::PageUp => b"\x1b[5~",
        Key::PageDown => b"\x1b[6~",
        Key::F1 => b"\x1bOP",
        Key::F2 => b"\x1bOQ",
        Key::F3 => b"\x1bOR",
        Key::F4 => b"\x1bOS",
        Key::F5 => b"\x1b[15~",
        Key::F6 => b"\x1b[17~",
        Key::F7 => b"\x1b[18~",
        Key::F8 => b"\x1b[19~",
        Key::F9 => b"\x1b[20~",
        Key::F10 => b"\x1b[21~",
        Key::F11 => b"\x1b[23~",
        Key::F12 => b"\x1b[24~",
        _ => return None,
    };
    Some(sequence.to_vec())
}

const fn rgb(color: Rgb) -> Color32 {
    Color32::from_rgb(color.0, color.1, color.2)
}
