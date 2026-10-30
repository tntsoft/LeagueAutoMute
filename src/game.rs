use serde::Deserialize;
use std::time::Duration;

const GAMESTATS_URL: &str = "https://127.0.0.1:2999/liveclientdata/gamestats";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStats {
    #[serde(default)]
    pub game_mode: String,
    #[serde(default)]
    pub game_time: f64,
    #[serde(default)]
    pub map_name: String,
}

/// Client for Riot's local Live Client Data API, which only listens while a
/// game is running. It uses a self-signed certificate, so verification is off.
pub struct LiveClient {
    http: reqwest::blocking::Client,
}

impl LiveClient {
    pub fn new() -> Result<Self, String> {
        let http = reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_millis(1500))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self { http })
    }

    /// `Ok(Some(stats))` while a game is loaded, `Ok(None)` when the API is not
    /// reachable (no game running), `Err` for anything unexpected.
    pub fn game_stats(&self) -> Result<Option<GameStats>, String> {
        let resp = match self.http.get(GAMESTATS_URL).send() {
            Ok(r) => r,
            // Connection refused / timed out: no game process is running.
            Err(e) if e.is_connect() || e.is_timeout() || e.is_request() => return Ok(None),
            Err(e) => return Err(e.to_string()),
        };
        if !resp.status().is_success() {
            // The API returns errors while the loading screen is up.
            return Ok(None);
        }
        resp.json::<GameStats>()
            .map(Some)
            .map_err(|e| format!("bad gamestats payload: {e}"))
    }
}
