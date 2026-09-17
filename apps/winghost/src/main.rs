#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;

use config::{AppConfig, ThemeChoice};
use eframe::egui::{
    self, Color32, Event, FontFamily, FontId, Key, RichText, Stroke, TextFormat, text::LayoutJob,
};
use pty_windows::{Session, ShellProfile};
use renderer::{Rgb, Theme};
use std::time::Duration;
use terminal_core::{ScreenSnapshot, Terminal};

const INITIAL_COLUMNS: u16 = 100;
const INITIAL_ROWS: u16 = 30;
const MAX_PANES: usize = 4;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WinGhost")
            .with_inner_size([1080.0, 720.0])
            .with_min_inner_size([640.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WinGhost",
        options,
        Box::new(|context| Ok(Box::new(WinGhostApp::new(context)))),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SplitDirection {
    Vertical,
    Horizontal,
}

struct Pane {
    terminal: Terminal,
    session: Option<Session>,
    profile: ShellProfile,
    status: String,
    size: (u16, u16),
    scrollback_lines: usize,
}

impl Pane {
    fn new(profile: ShellProfile, scrollback_lines: usize) -> Self {
        let mut pane = Self {
            terminal: Terminal::with_scrollback(INITIAL_COLUMNS, INITIAL_ROWS, scrollback_lines),
            session: None,
            profile,
            status: String::new(),
            size: (INITIAL_COLUMNS, INITIAL_ROWS),
            scrollback_lines,
        };
        pane.restart();
        pane
    }

    fn restart(&mut self) {
        self.terminal = Terminal::with_scrollback(self.size.0, self.size.1, self.scrollback_lines);
        match Session::spawn_profile(self.profile, self.size.0, self.size.1) {
            Ok(session) => {
                self.session = Some(session);
                self.profile.display_name().clone_into(&mut self.status);
            }
            Err(error) => {
                self.session = None;
                self.status = format!("Could not start {}: {error}", self.profile.display_name());
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
        self.terminal.scroll_to_bottom();
        if let Some(session) = &self.session
            && let Err(error) = session.send(bytes)
        {
            self.status = error.to_string();
        }
    }

    fn resize(&mut self, available: egui::Vec2, font_size: f32) {
        let cell_width = font_size * 0.58;
        let cell_height = font_size * 1.27;
        let columns = bounded_cell_count(available.x, cell_width, 12, 300);
        let rows = bounded_cell_count(available.y, cell_height, 4, 150);
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

struct TerminalTab {
    title: String,
    panes: Vec<Pane>,
    active_pane: usize,
    split: SplitDirection,
}

impl TerminalTab {
    fn new(number: usize, profile: ShellProfile, scrollback_lines: usize) -> Self {
        Self {
            title: format!("Tab {number}"),
            panes: vec![Pane::new(profile, scrollback_lines)],
            active_pane: 0,
            split: SplitDirection::Vertical,
        }
    }
}

struct WinGhostApp {
    tabs: Vec<TerminalTab>,
    active_tab: usize,
    config: AppConfig,
    show_settings: bool,
    notice: String,
}

impl WinGhostApp {
    fn new(context: &eframe::CreationContext<'_>) -> Self {
        let config = AppConfig::load();
        apply_visuals(&context.egui_ctx, config.theme);
        let profile = ShellProfile::from_id(&config.default_profile);
        Self {
            tabs: vec![TerminalTab::new(1, profile, config.scrollback_lines)],
            active_tab: 0,
            config,
            show_settings: false,
            notice: String::new(),
        }
    }

    fn active_pane_mut(&mut self) -> &mut Pane {
        let tab = &mut self.tabs[self.active_tab];
        &mut tab.panes[tab.active_pane]
    }

    fn active_pane(&self) -> &Pane {
        let tab = &self.tabs[self.active_tab];
        &tab.panes[tab.active_pane]
    }

    fn new_tab(&mut self) {
        let profile = ShellProfile::from_id(&self.config.default_profile);
        self.tabs.push(TerminalTab::new(
            self.tabs.len() + 1,
            profile,
            self.config.scrollback_lines,
        ));
        self.active_tab = self.tabs.len() - 1;
    }

    fn close_tab(&mut self) {
        if self.tabs.len() == 1 {
            self.active_pane_mut().restart();
            return;
        }
        self.tabs.remove(self.active_tab);
        self.active_tab = self.active_tab.min(self.tabs.len() - 1);
    }

    fn split_active(&mut self, direction: SplitDirection) {
        let profile = self.active_pane().profile;
        let scrollback_lines = self.config.scrollback_lines;
        let tab = &mut self.tabs[self.active_tab];
        if tab.panes.len() >= MAX_PANES {
            self.notice = format!("A tab supports up to {MAX_PANES} panes");
            return;
        }
        tab.split = direction;
        tab.panes.push(Pane::new(profile, scrollback_lines));
        tab.active_pane = tab.panes.len() - 1;
    }

    fn close_active_pane(&mut self) {
        let tab = &mut self.tabs[self.active_tab];
        if tab.panes.len() == 1 {
            tab.panes[0].restart();
            return;
        }
        tab.panes.remove(tab.active_pane);
        tab.active_pane = tab.active_pane.min(tab.panes.len() - 1);
    }

    fn receive_output(&mut self) {
        for tab in &mut self.tabs {
            for pane in &mut tab.panes {
                pane.receive_output();
            }
        }
    }

    fn handle_input(&mut self, context: &egui::Context) {
        if self.show_settings {
            return;
        }
        let events = context.input(|input| input.events.clone());
        for event in events {
            match event {
                Event::Text(text) => self.active_pane_mut().send(text.into_bytes()),
                Event::Paste(text) => self
                    .active_pane_mut()
                    .send(text.replace('\n', "\r\n").into_bytes()),
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => self.handle_key(key, modifiers),
                _ => {}
            }
        }
    }

    fn handle_key(&mut self, key: Key, modifiers: egui::Modifiers) {
        if modifiers.ctrl && modifiers.shift {
            match key {
                Key::T => self.new_tab(),
                Key::W => self.close_tab(),
                Key::E => self.split_active(SplitDirection::Vertical),
                Key::O => self.split_active(SplitDirection::Horizontal),
                _ => {}
            }
            return;
        }
        if modifiers.shift {
            match key {
                Key::PageUp => self.active_pane_mut().terminal.scroll_up(10),
                Key::PageDown => self.active_pane_mut().terminal.scroll_down(10),
                Key::Home => self.active_pane_mut().terminal.scroll_up(10_000),
                Key::End => self.active_pane_mut().terminal.scroll_to_bottom(),
                _ => {
                    if let Some(bytes) = key_bytes(key, modifiers) {
                        self.active_pane_mut().send(bytes);
                    }
                }
            }
            return;
        }
        if let Some(bytes) = key_bytes(key, modifiers) {
            self.active_pane_mut().send(bytes);
        }
    }

    fn show_settings_window(&mut self, context: &egui::Context) {
        if !self.show_settings {
            return;
        }
        let mut open = true;
        let mut save = false;
        egui::Window::new("WinGhost settings")
            .open(&mut open)
            .resizable(false)
            .show(context, |ui| {
                ui.label("Default shell profile");
                let selected = ShellProfile::from_id(&self.config.default_profile);
                egui::ComboBox::from_id_salt("default-profile")
                    .selected_text(selected.display_name())
                    .show_ui(ui, |ui| {
                        for profile in ShellProfile::ALL {
                            if profile.is_available() {
                                ui.selectable_value(
                                    &mut self.config.default_profile,
                                    profile.id().to_owned(),
                                    profile.display_name(),
                                );
                            }
                        }
                    });
                ui.add_space(8.0);
                ui.label("Font size");
                ui.add(egui::Slider::new(&mut self.config.font_size, 11.0..=24.0).suffix(" px"));
                ui.add_space(8.0);
                ui.label("Theme");
                egui::ComboBox::from_id_salt("theme")
                    .selected_text(self.config.theme.name())
                    .show_ui(ui, |ui| {
                        for theme in ThemeChoice::ALL {
                            ui.selectable_value(&mut self.config.theme, theme, theme.name());
                        }
                    });
                ui.add_space(8.0);
                ui.label("Scrollback lines");
                ui.add(
                    egui::Slider::new(&mut self.config.scrollback_lines, 1_000..=50_000)
                        .logarithmic(true),
                );
                ui.add_space(12.0);
                if ui.button("Save settings").clicked() {
                    save = true;
                }
                ui.small("New profile and scrollback settings apply to new sessions.");
            });
        self.show_settings = open;
        if save {
            apply_visuals(context, self.config.theme);
            match self.config.save() {
                Ok(path) => self.notice = format!("Saved settings to {}", path.display()),
                Err(error) => self.notice = error,
            }
        }
    }

    fn show_tabs(&mut self, ui: &mut egui::Ui) {
        let mut selected = None;
        ui.horizontal(|ui| {
            for (index, tab) in self.tabs.iter().enumerate() {
                let label = format!("{}  [{}]", tab.title, tab.panes.len());
                if ui
                    .selectable_label(index == self.active_tab, label)
                    .clicked()
                {
                    selected = Some(index);
                }
            }
            if ui
                .button("+")
                .on_hover_text("New tab (Ctrl+Shift+T)")
                .clicked()
            {
                selected = Some(self.tabs.len());
            }
        });
        if selected == Some(self.tabs.len()) {
            self.new_tab();
        } else if let Some(index) = selected {
            self.active_tab = index;
        }
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui, context: &egui::Context) {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("WinGhost")
                    .strong()
                    .color(Color32::from_rgb(110, 168, 254)),
            );
            ui.separator();
            ui.label(self.active_pane().profile.display_name());
            ui.label(format!(
                "{} × {}",
                self.active_pane().size.0,
                self.active_pane().size.1
            ));
            let offset = self.active_pane().terminal.scrollback_offset();
            if offset > 0 {
                ui.label(format!("Scrollback: {offset}"));
            }
            ui.separator();
            if ui.button("Split ↔").clicked() {
                self.split_active(SplitDirection::Vertical);
            }
            if ui.button("Split ↕").clicked() {
                self.split_active(SplitDirection::Horizontal);
            }
            if ui.button("Close pane").clicked() {
                self.close_active_pane();
            }
            if ui.button("Close tab").clicked() {
                self.close_tab();
            }
            if ui.button("↑ History").clicked() {
                self.active_pane_mut().terminal.scroll_up(10);
            }
            if ui.button("↓ Live").clicked() {
                self.active_pane_mut().terminal.scroll_to_bottom();
            }
            if ui.button("Copy").clicked() {
                context.copy_text(self.active_pane().terminal.contents());
            }
            if ui.button("Settings").clicked() {
                self.show_settings = true;
            }
            if !self.notice.is_empty() {
                ui.separator();
                ui.label(&self.notice);
            }
        });
    }

    fn show_active_tab(&mut self, ui: &mut egui::Ui) {
        let theme = self.config.theme.theme();
        let font_size = self.config.font_size;
        let tab = &mut self.tabs[self.active_tab];
        let mut clicked = None;
        match tab.split {
            SplitDirection::Vertical => {
                ui.columns(tab.panes.len(), |columns| {
                    for (index, (column, pane)) in
                        columns.iter_mut().zip(&mut tab.panes).enumerate()
                    {
                        if show_pane(column, pane, theme, font_size, index == tab.active_pane) {
                            clicked = Some(index);
                        }
                    }
                });
            }
            SplitDirection::Horizontal => {
                let pane_count = u16::try_from(tab.panes.len()).unwrap_or(1);
                let pane_height = ui.available_height() / f32::from(pane_count);
                let width = ui.available_width();
                for (index, pane) in tab.panes.iter_mut().enumerate() {
                    ui.allocate_ui(egui::vec2(width, pane_height), |pane_ui| {
                        if show_pane(pane_ui, pane, theme, font_size, index == tab.active_pane) {
                            clicked = Some(index);
                        }
                    });
                }
            }
        }
        if let Some(index) = clicked {
            tab.active_pane = index;
        }
    }
}

impl eframe::App for WinGhostApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.receive_output();
        self.handle_input(context);
        context.request_repaint_after(Duration::from_millis(16));

        egui::TopBottomPanel::top("tabs-and-toolbar").show(context, |ui| {
            self.show_tabs(ui);
            ui.separator();
            self.show_toolbar(ui, context);
        });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(rgb(self.config.theme.theme().background))
                    .inner_margin(6.0),
            )
            .show(context, |ui| self.show_active_tab(ui));
        self.show_settings_window(context);
    }
}

fn show_pane(
    ui: &mut egui::Ui,
    pane: &mut Pane,
    theme: Theme,
    font_size: f32,
    active: bool,
) -> bool {
    let stroke = if active {
        Stroke::new(2.0, rgb(theme.cursor))
    } else {
        Stroke::new(1.0, Color32::from_gray(45))
    };
    let response = egui::Frame::NONE
        .fill(rgb(theme.background))
        .stroke(stroke)
        .inner_margin(6.0)
        .show(ui, |ui| {
            pane.resize(ui.available_size(), font_size);
            let snapshot = pane.terminal.snapshot();
            let job = terminal_layout(&snapshot, theme, font_size);
            ui.add(egui::Label::new(job).selectable(false));
        })
        .response;
    let response = response.interact(egui::Sense::click());
    let clicked = response.clicked();
    claim_terminal_focus(&response, clicked);
    clicked
}

fn claim_terminal_focus(_response: &egui::Response, _clicked: bool) {}

fn terminal_layout(snapshot: &ScreenSnapshot, theme: Theme, font_size: f32) -> LayoutJob {
    let line_height = font_size * 1.27;
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
                    font_id: FontId::new(font_size, FontFamily::Monospace),
                    color: rgb(foreground),
                    background: rgb(background),
                    italics: cell.style.italic(),
                    underline: if cell.style.underline() {
                        Stroke::new(1.0, rgb(foreground))
                    } else {
                        Stroke::NONE
                    },
                    line_height: Some(line_height),
                    ..Default::default()
                },
            );
        }
        if row + 1 < snapshot.rows {
            job.append(
                "\n",
                0.0,
                TextFormat {
                    font_id: FontId::new(font_size, FontFamily::Monospace),
                    color: rgb(theme.foreground),
                    line_height: Some(line_height),
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

fn apply_visuals(context: &egui::Context, theme: ThemeChoice) {
    if theme == ThemeChoice::Light {
        context.set_visuals(egui::Visuals::light());
    } else {
        context.set_visuals(egui::Visuals::dark());
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn bounded_cell_count(available: f32, cell_size: f32, minimum: u16, maximum: u16) -> u16 {
    (available / cell_size)
        .floor()
        .clamp(f32::from(minimum), f32::from(maximum)) as u16
}

const fn rgb(color: Rgb) -> Color32 {
    Color32::from_rgb(color.0, color.1, color.2)
}

#[cfg(test)]
mod tests {
    use super::claim_terminal_focus;
    use eframe::egui;

    #[test]
    fn clicking_terminal_claims_keyboard_focus() {
        egui::__run_test_ui(|ui| {
            let response = ui.allocate_response(egui::vec2(320.0, 200.0), egui::Sense::click());
            claim_terminal_focus(&response, true);
            assert!(ui.memory(|memory| memory.has_focus(response.id)));
        });
    }
}
