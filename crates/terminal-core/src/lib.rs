//! Platform-independent terminal parsing and screen state.

/// A color emitted by the terminal parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalColor {
    Default,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

impl From<vt100::Color> for TerminalColor {
    fn from(color: vt100::Color) -> Self {
        match color {
            vt100::Color::Default => Self::Default,
            vt100::Color::Idx(index) => Self::Indexed(index),
            vt100::Color::Rgb(red, green, blue) => Self::Rgb(red, green, blue),
        }
    }
}

/// A renderer-neutral terminal cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub contents: String,
    pub foreground: TerminalColor,
    pub background: TerminalColor,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
    pub wide_continuation: bool,
}

/// An immutable renderer-facing copy of the visible terminal screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenSnapshot {
    pub columns: u16,
    pub rows: u16,
    pub cells: Vec<Cell>,
    pub cursor: (u16, u16),
    pub cursor_hidden: bool,
}

impl ScreenSnapshot {
    #[must_use]
    pub fn cell(&self, row: u16, column: u16) -> Option<&Cell> {
        let index = usize::from(row)
            .checked_mul(usize::from(self.columns))?
            .checked_add(usize::from(column))?;
        self.cells.get(index)
    }
}

/// A VT-compatible terminal parser with scrollback.
pub struct Terminal {
    parser: vt100::Parser,
}

impl Terminal {
    /// Creates an empty terminal grid with the requested scrollback capacity.
    #[must_use]
    pub fn new(columns: u16, rows: u16) -> Self {
        Self {
            parser: vt100::Parser::new(rows, columns, 10_000),
        }
    }

    /// Feeds bytes received from the pseudoconsole into the state machine.
    pub fn process(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    /// Returns the terminal width and height in character cells.
    #[must_use]
    pub fn size(&self) -> (u16, u16) {
        let (rows, columns) = self.parser.screen().size();
        (columns, rows)
    }

    /// Changes the terminal grid dimensions.
    pub fn resize(&mut self, columns: u16, rows: u16) {
        self.parser.set_size(rows, columns);
    }

    /// Returns the visible screen contents without terminal formatting codes.
    #[must_use]
    pub fn contents(&self) -> String {
        self.parser.screen().contents()
    }

    /// Copies visible screen state into a renderer-neutral snapshot.
    #[must_use]
    pub fn snapshot(&self) -> ScreenSnapshot {
        let screen = self.parser.screen();
        let (rows, columns) = screen.size();
        let mut cells = Vec::with_capacity(usize::from(rows) * usize::from(columns));

        for row in 0..rows {
            for column in 0..columns {
                let cell = screen.cell(row, column);
                cells.push(cell.map_or_else(empty_cell, |cell| Cell {
                    contents: cell.contents(),
                    foreground: cell.fgcolor().into(),
                    background: cell.bgcolor().into(),
                    bold: cell.bold(),
                    italic: cell.italic(),
                    underline: cell.underline(),
                    inverse: cell.inverse(),
                    wide_continuation: cell.is_wide_continuation(),
                }));
            }
        }

        ScreenSnapshot {
            columns,
            rows,
            cells,
            cursor: screen.cursor_position(),
            cursor_hidden: screen.hide_cursor(),
        }
    }
}

fn empty_cell() -> Cell {
    Cell {
        contents: String::new(),
        foreground: TerminalColor::Default,
        background: TerminalColor::Default,
        bold: false,
        italic: false,
        underline: false,
        inverse: false,
        wide_continuation: false,
    }
}

#[cfg(test)]
mod tests {
    use super::Terminal;

    #[test]
    fn resize_updates_dimensions() {
        let mut terminal = Terminal::new(80, 24);
        terminal.resize(120, 40);

        assert_eq!(terminal.size(), (120, 40));
    }

    #[test]
    fn parses_text_colors_and_cursor_position() {
        let mut terminal = Terminal::new(80, 24);
        terminal.process(b"hello\x1b[31m!\x1b[0m");
        let snapshot = terminal.snapshot();

        assert_eq!(terminal.contents().lines().next(), Some("hello!"));
        assert_eq!(snapshot.cursor, (0, 6));
        assert_eq!(
            snapshot.cell(0, 5).unwrap().foreground,
            super::TerminalColor::Indexed(1)
        );
    }
}
