use renderer::{Rgb, Theme};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeChoice {
    Midnight,
    Light,
    Matrix,
}

impl ThemeChoice {
    pub const ALL: [Self; 3] = [Self::Midnight, Self::Light, Self::Matrix];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Midnight => "Midnight",
            Self::Light => "Light",
            Self::Matrix => "Matrix",
        }
    }

    pub const fn theme(self) -> Theme {
        match self {
            Self::Midnight => Theme {
                foreground: Rgb(204, 211, 224),
                background: Rgb(13, 17, 23),
                cursor: Rgb(110, 168, 254),
            },
            Self::Light => Theme {
                foreground: Rgb(31, 35, 40),
                background: Rgb(246, 248, 250),
                cursor: Rgb(9, 105, 218),
            },
            Self::Matrix => Theme {
                foreground: Rgb(95, 255, 135),
                background: Rgb(3, 18, 8),
                cursor: Rgb(190, 255, 205),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub default_profile: String,
    pub font_size: f32,
    pub theme: ThemeChoice,
    pub scrollback_lines: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_profile: "powershell7".to_owned(),
            font_size: 15.0,
            theme: ThemeChoice::Midnight,
            scrollback_lines: 10_000,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let Some(path) = config_path() else {
            return Self::default();
        };
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<PathBuf, String> {
        let path = config_path().ok_or_else(|| "APPDATA is not available".to_owned())?;
        let parent = path
            .parent()
            .ok_or_else(|| "Invalid configuration path".to_owned())?;
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create settings folder: {error}"))?;
        let text = toml::to_string_pretty(self)
            .map_err(|error| format!("Could not serialize settings: {error}"))?;
        std::fs::write(&path, text)
            .map_err(|error| format!("Could not save settings: {error}"))?;
        Ok(path)
    }
}

fn config_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|directory| directory.join("WinGhost").join("config.toml"))
}
