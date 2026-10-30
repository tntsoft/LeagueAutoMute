# League Auto Mute

A tiny Windows background tool that types `/mute enemy` into League of Legends chat
once per match, automatically, so you never have to remember it.

## How it works

1. Polls Riot's local **Live Client Data API** (`https://127.0.0.1:2999/liveclientdata/gamestats`),
   which only answers while a game is running.
2. Waits until the in-game clock passes `min_game_time_secs` (HUD finished loading)
   **and** the game window (`League of Legends (TM) Client`) is the foreground window.
3. Sends `Enter`, types the command, sends `Enter` — once per match.
4. When the API stops answering (game over), it resets and waits for the next match.

Works for every mode, ARAM Mayhem included, because it keys off the game process, not the queue.

## Build

```bash
cargo build --release
```

The binary is `target\release\league_auto_mute.exe` (single file, no runtime needed).

## Run

Double-click the exe, or from a terminal:

```bash
.\target\release\league_auto_mute.exe
```

Leave the console window open (minimised is fine). It logs what it is doing:

```
[14:51:20] waiting for a League of Legends match...
[15:02:11] game detected: mode=ARAM map=Howling Abyss time=3s
[15:02:14] sent "/mute enemy"
[15:02:14] done for this match; waiting for it to end
[15:21:40] game ended
```

Flags:

| Flag | Meaning |
|------|---------|
| `--dry-run` / `-n` | Detect matches and log, but never send keystrokes. Good for a first test. |
| `--config <path>` / `-c` | Use a different config file (default: `config.toml` next to the exe). |

## Config

On first run a `config.toml` is written next to the exe:

```toml
commands = ["/mute enemy"]          # typed in order, once per match
poll_interval_ms = 2000
min_game_time_secs = 5.0            # wait for the HUD to load
window_title = "League of Legends (TM) Client"
require_foreground = true           # only type while the game is focused
key_delay_ms = 25
chat_open_delay_ms = 200
between_commands_ms = 400
```

Want more? e.g. `commands = ["/mute enemy", "/mute all"]` — each is sent as its own chat line.

If the game ever swallows the first keystrokes, raise `chat_open_delay_ms` to ~400.

## Start automatically with Windows

Run once from the project folder (PowerShell):

```bash
powershell -ExecutionPolicy Bypass -File .\install-startup.ps1
```

This drops a shortcut into your Startup folder that launches the exe minimised at login.
To remove it, delete `League Auto Mute.lnk` from `shell:startup`.

## Caveats

- It presses `Enter` blind. If you happen to have the chat box already open with text in it
  at the exact moment it fires (a few seconds into the game), that text gets sent. Unlikely but possible.
- Riot's rules on third-party tools restrict automation that gives a gameplay advantage.
  A one-shot chat command is about as benign as it gets, but it is still an external program
  sending input to the game. Use at your own discretion.
- Windows only (uses `SendInput` and window-title checks from the Win32 API).
