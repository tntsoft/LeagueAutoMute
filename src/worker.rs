//! Background thread: watches for a running match and types the chat command.

use crate::app::App;
use crate::config::Mode;
use crate::game::LiveClient;
use crate::input;
use crate::ui::log;
use std::thread::sleep;
use std::time::Duration;

/// Open chat, type the command, send it.
fn send_command(cmd: &str, key_delay: Duration, open_delay: Duration) -> Result<(), String> {
    input::press_enter()?;
    sleep(open_delay);
    input::type_text(cmd, key_delay)?;
    sleep(open_delay);
    input::press_enter()
}

pub fn run(app: &'static App) {
    let client = match LiveClient::new() {
        Ok(c) => c,
        Err(e) => {
            log(format!("fatal: could not create HTTP client: {e}"));
            return;
        }
    };

    let (poll, min_time, title, require_fg, key_delay, open_delay) = {
        let c = app.config.lock().unwrap();
        (
            Duration::from_millis(c.poll_interval_ms),
            c.min_game_time_secs,
            c.window_title.clone(),
            c.require_foreground,
            Duration::from_millis(c.key_delay_ms),
            Duration::from_millis(c.chat_open_delay_ms),
        )
    };

    log(format!("mode: {}  ({})", app.mode().label(), app.mode().command()));
    if app.dry_run {
        log("dry run: no keystrokes will be sent");
    }
    log("waiting for a League of Legends match...");

    let mut in_game = false;
    // Mode whose command has already been sent in the current match.
    let mut sent_mode: Option<Mode> = None;
    let mut focus_wait_logged = false;

    loop {
        let stats = match client.game_stats() {
            Ok(s) => s,
            Err(e) => {
                log(format!("api error: {e}"));
                sleep(poll);
                continue;
            }
        };

        let Some(stats) = stats else {
            if in_game {
                log("game ended");
                in_game = false;
                sent_mode = None;
                focus_wait_logged = false;
            }
            sleep(poll);
            continue;
        };

        if !in_game {
            in_game = true;
            log(format!(
                "game detected: mode={} map={} time={:.0}s",
                stats.game_mode, stats.map_name, stats.game_time
            ));
        }

        let mode = app.mode();
        if sent_mode == Some(mode) {
            sleep(poll);
            continue;
        }

        if stats.game_time < min_time {
            sleep(Duration::from_millis(500));
            continue;
        }

        if require_fg {
            let fg = input::foreground_title();
            if fg != title {
                if !focus_wait_logged {
                    let hint = if input::window_exists(&title) {
                        "game window is not focused"
                    } else {
                        "no window with the configured title found"
                    };
                    log(format!("{hint}; waiting (foreground: \"{fg}\")"));
                    focus_wait_logged = true;
                }
                sleep(Duration::from_millis(500));
                continue;
            }
        }

        let cmd = mode.command();
        if app.dry_run {
            log(format!("dry run: would type {cmd}"));
        } else {
            log(format!("typing {cmd}"));
            match send_command(cmd, key_delay, open_delay) {
                Ok(()) => log(format!("sent {cmd}")),
                Err(e) => log(format!("failed to send {cmd}: {e}")),
            }
        }
        sent_mode = Some(mode);
        focus_wait_logged = false;
        log("done for this match (switching mode in the tray re-sends)");
        sleep(poll);
    }
}
