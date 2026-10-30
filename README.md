# League Auto Mute

A tiny Windows tray app that types `/mute enemy` (or `/mute all`) into League of Legends
chat once per match, automatically, so you never have to remember it.

## How it works

1. Polls Riot's local **Live Client Data API** (`https://127.0.0.1:2999/liveclientdata/gamestats`),
   which only answers while a game is running.
2. Waits until the in-game clock passes `min_game_time_secs` (HUD finished loading)
   **and** the game window (`League of Legends (TM) Client`) is the foreground window.
3. Sends `Enter`, types the command, sends `Enter` — once per match.
4. When the API stops answering (game over), it resets and waits for the next match.

Works for every mode, ARAM Mayhem included, because it keys off the game process, not the queue.

## Using it

Run `league_auto_mute.exe`. A console-style window shows what it is doing:

```
[15:18:22] League Auto Mute started. Closing this window hides it to the tray.
[15:18:22] mode: Mute Enemy  (/mute enemy)
[15:18:22] waiting for a League of Legends match...
[15:18:22] game detected: mode=KIWI map=Map12 time=4s
[15:18:23] typing /mute enemy
[15:18:23] sent /mute enemy
```

- **Mode menu** (top of the window): `Mute Enemy` sends `/mute enemy` each match (default),
  `Mute All` sends `/mute all`. The window title shows the current mode.
- **Closing the window (X) hides it to the tray.** The app keeps running.
- **Left-click the tray icon** to bring the window back.
- **Right-click the tray icon** for the same options plus `Show Console`.

Switching mode is saved to `config.toml` and takes effect immediately. If a match is
already running, the new command is sent right away.

> **Can't see the tray icon?** Windows 11 hides new tray icons behind the `^` arrow on the
> taskbar. Click the arrow to find it, or drag it onto the taskbar to keep it visible
> (Settings → Personalization → Taskbar → Other system tray icons).

Flags:

| Flag | Meaning |
|------|---------|
| `--minimized` / `-m` | Start hidden in the tray (used by the startup shortcut). |
| `--dry-run` / `-n` | Detect matches and log, but never send keystrokes. Good for a first test. |
| `--config <path>` / `-c` | Use a different config file (default: `config.toml` next to the exe). |

## Start automatically with Windows

On by default. The first time you run the app it registers itself in the per-user Startup
list (`HKCU\...\CurrentVersion\Run`) so it launches minimised to the tray at login.
No admin rights needed.

To turn it off or on: **App → Start with Windows** in the window, or the same entry in the
tray menu. The choice is saved as `start_with_windows` in `config.toml`.

If you move the exe to another folder, just run it once from the new location and the
Startup entry is updated automatically.

## Config

`config.toml` is created next to the exe on first run:

```toml
mode = "enemy"                      # or "all"; also switchable from the tray
start_with_windows = true           # register in the per-user Startup list
poll_interval_ms = 2000
min_game_time_secs = 5.0            # wait for the HUD to load
window_title = "League of Legends (TM) Client"
require_foreground = true           # only type while the game is focused
key_delay_ms = 25
chat_open_delay_ms = 200
```

If the game ever swallows the first keystrokes, raise `chat_open_delay_ms` to ~400.

## Build from source

```bash
cargo build --release
```

The binary is `target\release\league_auto_mute.exe` (single file, no runtime needed).

## Caveats

- It presses `Enter` blind. If you happen to have the chat box already open with text in it
  at the exact moment it fires (a few seconds into the game), that text gets sent. Unlikely but possible.
- Riot's rules on third-party tools restrict automation that gives a gameplay advantage.
  A one-shot chat command is about as benign as it gets, but it is still an external program
  sending input to the game. Use at your own discretion.
- Windows only (Win32 tray icon, `SendInput`, window-title checks).
