mod config;
mod game;
mod input;

use config::Config;
use game::LiveClient;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

fn log(msg: impl AsRef<str>) {
    println!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg.as_ref());
}

struct Args {
    config_path: PathBuf,
    dry_run: bool,
}

fn parse_args() -> Args {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    let mut args = Args {
        config_path: exe_dir.join("config.toml"),
        dry_run: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--dry-run" | "-n" => args.dry_run = true,
            "--config" | "-c" => {
                if let Some(p) = it.next() {
                    args.config_path = PathBuf::from(p);
                }
            }
            "--help" | "-h" => {
                println!(
                    "league_auto_mute [--config <path>] [--dry-run]\n\n\
                     Types the configured chat commands once per League of Legends match.\n\
                     --config   path to config.toml (default: next to the exe)\n\
                     --dry-run  detect games and log, but do not send any keystrokes"
                );
                std::process::exit(0);
            }
            other => eprintln!("ignoring unknown argument: {other}"),
        }
    }
    args
}

/// Open chat, type the command, send it.
fn send_command(cmd: &str, cfg: &Config) -> Result<(), String> {
    let key_delay = Duration::from_millis(cfg.key_delay_ms);
    let open_delay = Duration::from_millis(cfg.chat_open_delay_ms);
    input::press_enter()?;
    sleep(open_delay);
    input::type_text(cmd, key_delay)?;
    sleep(open_delay);
    input::press_enter()
}

fn main() {
    let args = parse_args();
    let cfg = match Config::load_or_create(&args.config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("config error: {e}");
            std::process::exit(1);
        }
    };
    let client = match LiveClient::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("http client error: {e}");
            std::process::exit(1);
        }
    };

    log(format!("config: {}", args.config_path.display()));
    log(format!("commands: {:?}", cfg.commands));
    if args.dry_run {
        log("dry run: no keystrokes will be sent");
    }
    log("waiting for a League of Legends match...");

    let poll = Duration::from_millis(cfg.poll_interval_ms);
    // True once commands were sent for the current match; reset when the game ends.
    let mut done_this_game = false;
    let mut in_game = false;
    let mut waiting_for_focus_logged = false;

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
                done_this_game = false;
                waiting_for_focus_logged = false;
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

        if done_this_game {
            sleep(poll);
            continue;
        }

        if stats.game_time < cfg.min_game_time_secs {
            sleep(Duration::from_millis(500));
            continue;
        }

        if cfg.require_foreground {
            let fg = input::foreground_title();
            if fg != cfg.window_title {
                if !waiting_for_focus_logged {
                    let hint = if input::window_exists(&cfg.window_title) {
                        "game window is not focused"
                    } else {
                        "no window with the configured title found"
                    };
                    log(format!("{hint}; waiting (foreground: \"{fg}\")"));
                    waiting_for_focus_logged = true;
                }
                sleep(Duration::from_millis(500));
                continue;
            }
        }

        // Game is running, past the minimum time, and focused: send the commands.
        for (i, cmd) in cfg.commands.iter().enumerate() {
            if args.dry_run {
                log(format!("dry run: would type {cmd:?}"));
                continue;
            }
            match send_command(cmd, &cfg) {
                Ok(()) => log(format!("sent {cmd:?}")),
                Err(e) => log(format!("failed to send {cmd:?}: {e}")),
            }
            if i + 1 < cfg.commands.len() {
                sleep(Duration::from_millis(cfg.between_commands_ms));
            }
        }
        done_this_game = true;
        log("done for this match; waiting for it to end");
        sleep(poll);
    }
}
