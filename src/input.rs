//! Foreground-window check and keystroke injection (Windows only).

use std::thread::sleep;
use std::time::Duration;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    VK_RETURN,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetForegroundWindow, GetWindowTextW,
};

const SCAN_ENTER: u16 = 0x1C;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Title of the current foreground window, or an empty string.
pub fn foreground_title() -> String {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return String::new();
        }
        let mut buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        String::from_utf16_lossy(&buf[..len.max(0) as usize])
    }
}

/// True if a top-level window with exactly this title exists.
pub fn window_exists(title: &str) -> bool {
    let w = wide(title);
    unsafe { !FindWindowW(std::ptr::null(), w.as_ptr()).is_null() }
}

fn key_input(vk: u16, scan: u16, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send(inputs: &mut [INPUT]) -> Result<(), String> {
    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_mut_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        )
    };
    if sent as usize == inputs.len() {
        Ok(())
    } else {
        Err(format!(
            "SendInput injected {sent} of {} events (another program may be blocking input)",
            inputs.len()
        ))
    }
}

/// Press and release Enter.
pub fn press_enter() -> Result<(), String> {
    let mut inputs = [
        key_input(VK_RETURN, SCAN_ENTER, 0),
        key_input(VK_RETURN, SCAN_ENTER, KEYEVENTF_KEYUP),
    ];
    send(&mut inputs)
}

/// Type a string as Unicode key events, one character at a time.
pub fn type_text(text: &str, key_delay: Duration) -> Result<(), String> {
    for unit in text.encode_utf16() {
        let mut inputs = [
            key_input(0, unit, KEYEVENTF_UNICODE),
            key_input(0, unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP),
        ];
        send(&mut inputs)?;
        sleep(key_delay);
    }
    Ok(())
}
