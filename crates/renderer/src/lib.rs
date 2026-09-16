//! Theme and color resolution shared by renderer front ends.

use terminal_core::TerminalColor;

/// An RGB color independent of a particular rendering API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// The built-in dark theme used by the first Windows prototype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub foreground: Rgb,
    pub background: Rgb,
    pub cursor: Rgb,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            foreground: Rgb(204, 211, 224),
            background: Rgb(13, 17, 23),
            cursor: Rgb(110, 168, 254),
        }
    }
}

impl Theme {
    #[must_use]
    pub fn resolve(self, color: TerminalColor, is_background: bool) -> Rgb {
        match color {
            TerminalColor::Default if is_background => self.background,
            TerminalColor::Default => self.foreground,
            TerminalColor::Indexed(index) => indexed_color(index),
            TerminalColor::Rgb(red, green, blue) => Rgb(red, green, blue),
        }
    }
}

#[must_use]
fn indexed_color(index: u8) -> Rgb {
    const ANSI: [Rgb; 16] = [
        Rgb(30, 34, 42),
        Rgb(224, 85, 85),
        Rgb(152, 195, 121),
        Rgb(229, 192, 123),
        Rgb(97, 175, 239),
        Rgb(198, 120, 221),
        Rgb(86, 182, 194),
        Rgb(215, 218, 224),
        Rgb(92, 99, 112),
        Rgb(255, 107, 107),
        Rgb(182, 224, 146),
        Rgb(255, 215, 142),
        Rgb(119, 190, 255),
        Rgb(218, 145, 239),
        Rgb(111, 211, 225),
        Rgb(255, 255, 255),
    ];

    match index {
        0..=15 => ANSI[usize::from(index)],
        16..=231 => {
            let value = index - 16;
            let red = value / 36;
            let green = (value % 36) / 6;
            let blue = value % 6;
            Rgb(cube(red), cube(green), cube(blue))
        }
        232..=255 => {
            let gray = 8 + (index - 232) * 10;
            Rgb(gray, gray, gray)
        }
    }
}

const fn cube(value: u8) -> u8 {
    if value == 0 { 0 } else { 55 + value * 40 }
}
