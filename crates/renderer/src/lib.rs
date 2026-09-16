//! Renderer-facing data types. No GPU backend is selected yet.

use terminal_core::Terminal;

/// An immutable description of the terminal grid needed by a renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSnapshot {
    columns: u16,
    rows: u16,
}

impl RenderSnapshot {
    /// Captures renderer-visible dimensions from terminal state.
    #[must_use]
    pub const fn from_terminal(terminal: &Terminal) -> Self {
        Self {
            columns: terminal.columns(),
            rows: terminal.rows(),
        }
    }

    /// Returns the grid width in character cells.
    #[must_use]
    pub const fn columns(&self) -> u16 {
        self.columns
    }

    /// Returns the grid height in character cells.
    #[must_use]
    pub const fn rows(&self) -> u16 {
        self.rows
    }
}
