//! "Start with Windows" via the per-user Run registry key
//! (HKCU\Software\Microsoft\Windows\CurrentVersion\Run). No admin rights needed.

use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALUE_NAME: &str = "LeagueAutoMute";

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// The command line we register: `"<exe>" --minimized`.
fn command_line() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    Ok(format!("\"{}\" --minimized", exe.display()))
}

fn open_run_key(access: u32) -> Result<HKEY, String> {
    let mut hkey: HKEY = null_mut();
    let sub = wide(RUN_KEY);
    let rc = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, sub.as_ptr(), 0, access, &mut hkey) };
    if rc == ERROR_SUCCESS {
        Ok(hkey)
    } else {
        Err(format!("could not open HKCU\\{RUN_KEY} (error {rc})"))
    }
}

/// Current registered command line, if any.
fn current_value() -> Option<String> {
    let hkey = open_run_key(KEY_READ).ok()?;
    let name = wide(VALUE_NAME);
    let mut buf = vec![0u16; 2048];
    let mut size = (buf.len() * 2) as u32;
    let mut kind = 0u32;
    let rc = unsafe {
        RegQueryValueExW(
            hkey,
            name.as_ptr(),
            null(),
            &mut kind,
            buf.as_mut_ptr().cast(),
            &mut size,
        )
    };
    unsafe { RegCloseKey(hkey) };
    if rc != ERROR_SUCCESS || kind != REG_SZ {
        return None;
    }
    let len = (size as usize / 2).min(buf.len());
    let s = String::from_utf16_lossy(&buf[..len]);
    Some(s.trim_end_matches('\0').to_string())
}

/// True when the Run entry exists and points at this executable.
pub fn is_enabled() -> bool {
    let Some(value) = current_value() else {
        return false;
    };
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    value
        .to_lowercase()
        .contains(&exe.display().to_string().to_lowercase())
}

/// Register or remove the Run entry.
pub fn set_enabled(enabled: bool) -> Result<(), String> {
    let hkey = open_run_key(KEY_READ | KEY_WRITE)?;
    let name = wide(VALUE_NAME);
    let rc = if enabled {
        let data = wide(&command_line()?);
        unsafe {
            RegSetValueExW(
                hkey,
                name.as_ptr(),
                0,
                REG_SZ,
                data.as_ptr().cast(),
                (data.len() * 2) as u32,
            )
        }
    } else {
        unsafe { RegDeleteValueW(hkey, name.as_ptr()) }
    };
    unsafe { RegCloseKey(hkey) };
    if rc == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("registry write failed (error {rc})"))
    }
}
