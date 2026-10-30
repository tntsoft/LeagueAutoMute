//! Process-wide state shared between the UI thread and the worker thread.

use crate::config::{Config, Mode};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};

pub struct App {
    pub config_path: PathBuf,
    pub config: Mutex<Config>,
    mode: AtomicU8,
    pub dry_run: bool,
}

static APP: OnceLock<App> = OnceLock::new();

impl App {
    pub fn init(config_path: PathBuf, config: Config, dry_run: bool) -> &'static App {
        let app = App {
            config_path,
            mode: AtomicU8::new(config.mode.as_u8()),
            config: Mutex::new(config),
            dry_run,
        };
        APP.get_or_init(|| app)
    }

    pub fn get() -> &'static App {
        APP.get().expect("App::init must be called first")
    }

    pub fn mode(&self) -> Mode {
        Mode::from_u8(self.mode.load(Ordering::Relaxed))
    }

    pub fn start_with_windows(&self) -> bool {
        self.config.lock().unwrap().start_with_windows
    }

    /// Persist the start-with-Windows preference and apply it to the registry.
    pub fn set_start_with_windows(&self, enabled: bool) -> Result<(), String> {
        {
            let mut cfg = self.config.lock().unwrap();
            cfg.start_with_windows = enabled;
            cfg.save(&self.config_path)?;
        }
        crate::autostart::set_enabled(enabled)
    }

    /// Make the registry match the saved preference (called at startup).
    pub fn apply_start_with_windows(&self) -> Result<bool, String> {
        let enabled = self.start_with_windows();
        if enabled {
            crate::autostart::set_enabled(true)?;
        } else if crate::autostart::is_enabled() {
            crate::autostart::set_enabled(false)?;
        }
        Ok(enabled)
    }

    /// Switch mode and persist it. Returns an error message if saving failed.
    pub fn set_mode(&self, mode: Mode) -> Result<(), String> {
        self.mode.store(mode.as_u8(), Ordering::Relaxed);
        let mut cfg = self.config.lock().unwrap();
        cfg.mode = mode;
        cfg.save(&self.config_path)
    }
}
