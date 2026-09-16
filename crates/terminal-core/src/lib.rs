//! Platform-independent terminal state.

/// A deliberately small terminal model used to establish workspace boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Terminal {
    columns: u16,
    rows: u16,
}

impl Terminal {
    /// Creates an empty terminal grid.
    #[must_use]
    pub const fn new(columns: u16, rows: u16) -> Self {
        Self { columns, rows }
    }

    /// Returns the terminal width in character cells.
    #[must_use]
    pub const fn columns(&self) -> u16 {
        self.columns
    }

    /// Returns the terminal height in character cells.
    #[must_use]
    pub const fn rows(&self) -> u16 {
        self.rows
    }

    /// Changes the terminal grid dimensions.
    pub const fn resize(&mut self, columns: u16, rows: u16) {
        self.columns = columns;
        self.rows = rows;
    }
}

#[cfg(test)]
mod tests {
    use super::Terminal;

    #[test]
    fn resize_updates_dimensions() {
        let mut terminal = Terminal::new(80, 24);
        terminal.resize(120, 40);

        assert_eq!(terminal.columns(), 120);
        assert_eq!(terminal.rows(), 40);
    }
}
