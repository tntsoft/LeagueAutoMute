#![windows_subsystem = "windows"]

mod app;
mod config;
mod game;
mod icon;
mod input;
mod ui;
mod worker;

use app::App;
use config::Config;
use std::path::PathBuf;
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

struct Args {
    config_path: PathBuf,
    dry_run: bool,
    minimized: bool,
}

fn parse_args() -> Args {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    let mut args = Args {
        config_path: exe_dir.join("config.toml"),
        dry_run: false,
        minimized: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--dry-run" | "-n" => args.dry_run = true,
            "--minimized" | "-m" => args.minimized = true,
            "--config" | "-c" => {
                if let Some(p) = it.next() {
                    args.config_path = PathBuf::from(p);
                }
            }
            _ => {}
        }
    }
    args
}

fn fatal(msg: &str) -> ! {
    let text: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
    let title: Vec<u16> = "League Auto Mute".encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
    }
    std::process::exit(1);
}

fn main() {
    let args = parse_args();
    let config = match Config::load_or_create(&args.config_path) {
        Ok(c) => c,
        Err(e) => fatal(&format!("Config error: {e}")),
    };
    let app = App::init(args.config_path, config, args.dry_run);

    if let Err(e) = ui::init(args.minimized) {
        fatal(&format!("UI error: {e}"));
    }
    ui::log(format!("config: {}", app.config_path.display()));

    std::thread::spawn(move || worker::run(app));

    let code = ui::message_loop();
    std::process::exit(code);
}
