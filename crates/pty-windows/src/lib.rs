//! Windows pseudoconsole boundary.
//!
//! `ConPTY` calls will be added behind this crate's public API during Milestone 1.

use std::fmt;

/// Compile-time platform support reported by the scaffold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformSupport {
    Windows,
    Unsupported,
}

impl PlatformSupport {
    /// Returns support for the current compilation target.
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Unsupported
        }
    }
}

impl fmt::Display for PlatformSupport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Windows => formatter.write_str("Windows/ConPTY planned"),
            Self::Unsupported => formatter.write_str("unsupported host"),
        }
    }
}
