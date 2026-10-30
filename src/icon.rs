//! Loads the embedded icon resource (see `app.rc` / `build.rs`).

use std::ptr::null;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, LoadImageW, HICON, IMAGE_ICON, LR_DEFAULTCOLOR, LR_SHARED, SM_CXSMICON,
    SM_CYSMICON,
};

/// Resource id from `app.rc`.
const ICON_ID: usize = 1;

/// Load the embedded icon at the given pixel size (0 = the resource's default).
pub fn load(width: i32, height: i32) -> HICON {
    unsafe {
        LoadImageW(
            GetModuleHandleW(null()),
            ICON_ID as *const u16,
            IMAGE_ICON,
            width,
            height,
            LR_DEFAULTCOLOR | LR_SHARED,
        ) as HICON
    }
}

/// Icon sized for the notification area (16 px at 100 % DPI).
pub fn tray() -> HICON {
    unsafe { load(GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON)) }
}

/// Standard 32 px icon for the window class / title bar.
pub fn window() -> HICON {
    load(32, 32)
}
