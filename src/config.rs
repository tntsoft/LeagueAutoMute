use serde::{Deserialize, Serialize};
use std::path::Path;

/// Which chat command to send at the start of each match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Enemy,
    All,
}

impl Mode {
    pub fn command(self) -> &'static str {
        match self {
            Mode::Enemy => "/mute enemy",
            Mode::All => "/mute all",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Mode::Enemy => "Mute Enemy",
            Mode::All => "Mute All",
        }
    }

    pub fn as_u8(self) -> u8 {
        match self {
            Mode::Enemy => 0,
            Mode::All => 1,
        }
    }

    pub fn from_u8(v: u8) -> Mode {
        if v == 1 {
            Mode::All
        } else {
            Mode::Enemy
        }
    }
}

/// User-editable settings, stored as `config.toml` next to the executable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// `enemy` or `all`. Also switchable from the tray menu.
    pub mode: Mode,
    /// How often to poll the Live Client Data API when no game is running (ms).
    pub poll_interval_ms: u64,
    /// Do not type anything until the in-game clock has reached this many seconds.
    /// Gives the HUD time to finish loading after the loading screen.
    pub min_game_time_secs: f64,
    /// Exact title of the in-game window.
    pub window_title: String,
    /// Only type when the game window is the foreground window.
    pub require_foreground: bool,
    /// Delay between individual key presses (ms).
    pub key_delay_ms: u64,
    /// Delay between opening the chat box and typing / pressing Enter (ms).
    pub chat_open_delay_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Enemy,
            poll_interval_ms: 2000,
            min_game_time_secs: 5.0,
            window_title: "League of Legends (TM) Client".to_string(),
            require_foreground: true,
            key_delay_ms: 25,
            chat_open_delay_ms: 200,
        }
    }
}

const HEADER: &str = "# league_auto_mute configuration\n\
                      # mode is also switchable from the tray icon. Other changes need a restart.\n\n";

impl Config {
    /// Load the config from `path`, writing a default file first if none exists.
    pub fn load_or_create(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            let default = Config::default();
            default.save(path)?;
            return Ok(default);
        }
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("could not read {}: {e}", path.display()))?;
        toml::from_str(&text).map_err(|e| format!("invalid {}: {e}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let text = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, format!("{HEADER}{text}"))
            .map_err(|e| format!("could not write {}: {e}", path.display()))
    }
}
