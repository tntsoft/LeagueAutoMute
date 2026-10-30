use serde::{Deserialize, Serialize};
use std::path::Path;

/// User-editable settings, stored as `config.toml` next to the executable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Chat commands to type, in order, once per match.
    pub commands: Vec<String>,
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
    /// Delay between consecutive commands (ms).
    pub between_commands_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            commands: vec!["/mute enemy".to_string()],
            poll_interval_ms: 2000,
            min_game_time_secs: 5.0,
            window_title: "League of Legends (TM) Client".to_string(),
            require_foreground: true,
            key_delay_ms: 25,
            chat_open_delay_ms: 200,
            between_commands_ms: 400,
        }
    }
}

impl Config {
    /// Load the config from `path`, writing a default file first if none exists.
    pub fn load_or_create(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            let default = Config::default();
            let text = toml::to_string_pretty(&default).map_err(|e| e.to_string())?;
            let header = "# league_auto_mute configuration\n\
                          # Edit and restart the program to apply changes.\n\n";
            std::fs::write(path, format!("{header}{text}"))
                .map_err(|e| format!("could not write {}: {e}", path.display()))?;
            return Ok(default);
        }
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("could not read {}: {e}", path.display()))?;
        toml::from_str(&text).map_err(|e| format!("invalid {}: {e}", path.display()))
    }
}
