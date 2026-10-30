//! Win32 UI: a log window ("console") that hides to the tray on close, plus a
//! tray icon with a menu to pick the mute mode.

use crate::app::App;
use crate::config::Mode;
use crate::icon;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicIsize, Ordering};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontW, DeleteObject, HBRUSH, HFONT, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS,
    COLOR_WINDOW, DEFAULT_CHARSET, FF_MODERN, FIXED_PITCH, FW_NORMAL, OUT_DEFAULT_PRECIS,
};
use windows_sys::Win32::UI::Controls::{EM_REPLACESEL, EM_SCROLLCARET, EM_SETLIMITTEXT, EM_SETSEL};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::HiDpi::{
    GetDpiForWindow, SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
    DispatchMessageW, GetCursorPos, GetMessageW, GetWindowTextLengthW, LoadCursorW, MoveWindow,
    PostMessageW, PostQuitMessage, RegisterClassW, SendMessageW, SetForegroundWindow,
    SetWindowPos, SetWindowTextW, ShowWindow, TrackPopupMenu, TranslateMessage,
    CW_USEDEFAULT, ES_AUTOVSCROLL,
    ES_MULTILINE, ES_READONLY, IDC_ARROW, MF_CHECKED, MF_SEPARATOR, MF_STRING, MSG,
    SWP_NOACTIVATE, SWP_NOZORDER, SW_HIDE, SW_RESTORE, SW_SHOW, TPM_NONOTIFY, TPM_RETURNCMD,
    TPM_RIGHTBUTTON, WM_APP, WM_CLOSE, WM_CREATE, WM_DESTROY, WM_DPICHANGED, WM_LBUTTONDBLCLK,
    WM_LBUTTONUP, WM_NULL, WM_RBUTTONUP, WM_SETFONT, WM_SIZE, WNDCLASSW, WS_CHILD,
    WS_EX_CLIENTEDGE, WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_VSCROLL,
};

const WM_TRAY: u32 = WM_APP + 1;
const WM_LOG: u32 = WM_APP + 2;
const TRAY_ID: u32 = 1;

const ID_MODE_ENEMY: usize = 1;
const ID_MODE_ALL: usize = 2;
const ID_SHOW: usize = 3;
const ID_EXIT: usize = 4;

const APP_TITLE: &str = "League Auto Mute";

static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);
static EDIT_HWND: AtomicIsize = AtomicIsize::new(0);
static ICON: AtomicIsize = AtomicIsize::new(0);
static FONT: AtomicIsize = AtomicIsize::new(0);

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn main_hwnd() -> HWND {
    MAIN_HWND.load(Ordering::Acquire) as HWND
}

fn edit_hwnd() -> HWND {
    EDIT_HWND.load(Ordering::Acquire) as HWND
}

/// Append a timestamped line to the log window. Safe to call from any thread.
pub fn log(msg: impl AsRef<str>) {
    let line = format!(
        "[{}] {}\r\n",
        chrono::Local::now().format("%H:%M:%S"),
        msg.as_ref()
    );
    let hwnd = main_hwnd();
    if hwnd.is_null() {
        return;
    }
    let boxed = Box::into_raw(Box::new(line));
    let ok = unsafe { PostMessageW(hwnd, WM_LOG, 0, boxed as LPARAM) };
    if ok == 0 {
        // Queue full or window gone: reclaim the allocation.
        unsafe { drop(Box::from_raw(boxed)) };
    }
}

unsafe fn append_text(text: &str) {
    let edit = edit_hwnd();
    if edit.is_null() {
        return;
    }
    let mut len = GetWindowTextLengthW(edit);
    if len > 200_000 {
        SetWindowTextW(edit, wide("").as_ptr());
        len = 0;
    }
    SendMessageW(edit, EM_SETSEL, len as WPARAM, len as LPARAM);
    let w = wide(text);
    SendMessageW(edit, EM_REPLACESEL, 0, w.as_ptr() as LPARAM);
    SendMessageW(edit, EM_SCROLLCARET, 0, 0);
}

unsafe fn make_font(dpi: u32) -> HFONT {
    let height = -((10 * dpi as i32) / 72);
    CreateFontW(
        height,
        0,
        0,
        0,
        FW_NORMAL as _,
        0,
        0,
        0,
        DEFAULT_CHARSET as _,
        OUT_DEFAULT_PRECIS as _,
        CLIP_DEFAULT_PRECIS as _,
        CLEARTYPE_QUALITY as _,
        (FIXED_PITCH | FF_MODERN) as _,
        wide("Consolas").as_ptr(),
    )
}

unsafe fn apply_font(dpi: u32) {
    let old = FONT.swap(0, Ordering::AcqRel);
    if old != 0 {
        DeleteObject(old as _);
    }
    let font = make_font(dpi);
    FONT.store(font as isize, Ordering::Release);
    SendMessageW(edit_hwnd(), WM_SETFONT, font as WPARAM, 1);
}

unsafe fn tray_data(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = TRAY_ID;
    nid
}

unsafe fn set_tip(nid: &mut NOTIFYICONDATAW, mode: Mode) {
    let tip = wide(&format!("{APP_TITLE} - {}", mode.label()));
    let n = tip.len().min(nid.szTip.len());
    nid.szTip[..n].copy_from_slice(&tip[..n]);
    nid.szTip[nid.szTip.len() - 1] = 0;
}

unsafe fn tray_add(hwnd: HWND) {
    let mut nid = tray_data(hwnd);
    nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.uCallbackMessage = WM_TRAY;
    nid.hIcon = ICON.load(Ordering::Acquire) as _;
    set_tip(&mut nid, App::get().mode());
    Shell_NotifyIconW(NIM_ADD, &nid);
}

unsafe fn tray_update_tip(hwnd: HWND) {
    let mut nid = tray_data(hwnd);
    nid.uFlags = NIF_TIP;
    set_tip(&mut nid, App::get().mode());
    Shell_NotifyIconW(NIM_MODIFY, &nid);
}

unsafe fn tray_remove(hwnd: HWND) {
    let nid = tray_data(hwnd);
    Shell_NotifyIconW(NIM_DELETE, &nid);
}

unsafe fn show_main(hwnd: HWND) {
    ShowWindow(hwnd, SW_SHOW);
    ShowWindow(hwnd, SW_RESTORE);
    SetForegroundWindow(hwnd);
}

unsafe fn show_menu(hwnd: HWND) {
    let menu = CreatePopupMenu();
    let mode = App::get().mode();
    let checked = |m: Mode| if mode == m { MF_CHECKED } else { 0 };
    AppendMenuW(
        menu,
        MF_STRING | checked(Mode::Enemy),
        ID_MODE_ENEMY,
        wide(Mode::Enemy.label()).as_ptr(),
    );
    AppendMenuW(
        menu,
        MF_STRING | checked(Mode::All),
        ID_MODE_ALL,
        wide(Mode::All.label()).as_ptr(),
    );
    AppendMenuW(menu, MF_SEPARATOR, 0, null());
    AppendMenuW(menu, MF_STRING, ID_SHOW, wide("Show Console").as_ptr());
    AppendMenuW(menu, MF_SEPARATOR, 0, null());
    AppendMenuW(menu, MF_STRING, ID_EXIT, wide("Exit").as_ptr());

    let mut pt = POINT { x: 0, y: 0 };
    GetCursorPos(&mut pt);
    // Required so the menu closes when the user clicks elsewhere.
    SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(
        menu,
        TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_NONOTIFY,
        pt.x,
        pt.y,
        0,
        hwnd,
        null(),
    );
    PostMessageW(hwnd, WM_NULL, 0, 0);
    DestroyMenu(menu);
    handle_command(hwnd, cmd as usize);
}

unsafe fn handle_command(hwnd: HWND, id: usize) {
    match id {
        ID_MODE_ENEMY | ID_MODE_ALL => {
            let mode = if id == ID_MODE_ENEMY { Mode::Enemy } else { Mode::All };
            match App::get().set_mode(mode) {
                Ok(()) => log(format!("mode set to {} ({})", mode.label(), mode.command())),
                Err(e) => log(format!("mode set to {} but saving config failed: {e}", mode.label())),
            }
            tray_update_tip(hwnd);
        }
        ID_SHOW => show_main(hwnd),
        ID_EXIT => {
            DestroyWindow(hwnd);
        }
        _ => {}
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            MAIN_HWND.store(hwnd as isize, Ordering::Release);
            let hinst = GetModuleHandleW(null());
            let edit = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                wide("EDIT").as_ptr(),
                null(),
                WS_CHILD
                    | WS_VISIBLE
                    | WS_VSCROLL
                    | (ES_MULTILINE | ES_READONLY | ES_AUTOVSCROLL) as u32,
                0,
                0,
                0,
                0,
                hwnd,
                null_mut(),
                hinst,
                null(),
            );
            EDIT_HWND.store(edit as isize, Ordering::Release);
            SendMessageW(edit, EM_SETLIMITTEXT, 0, 0);
            apply_font(GetDpiForWindow(hwnd));
            tray_add(hwnd);
            0
        }
        WM_SIZE => {
            let w = (lparam & 0xFFFF) as i32;
            let h = ((lparam >> 16) & 0xFFFF) as i32;
            MoveWindow(edit_hwnd(), 0, 0, w, h, 1);
            0
        }
        WM_DPICHANGED => {
            apply_font((wparam & 0xFFFF) as u32);
            let rect = &*(lparam as *const RECT);
            SetWindowPos(
                hwnd,
                null_mut(),
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            0
        }
        WM_LOG => {
            let line = Box::from_raw(lparam as *mut String);
            append_text(&line);
            0
        }
        WM_TRAY => {
            match lparam as u32 {
                WM_LBUTTONUP | WM_LBUTTONDBLCLK => show_main(hwnd),
                WM_RBUTTONUP => show_menu(hwnd),
                _ => {}
            }
            0
        }
        // The X button hides to the tray instead of quitting.
        WM_CLOSE => {
            ShowWindow(hwnd, SW_HIDE);
            0
        }
        WM_DESTROY => {
            tray_remove(hwnd);
            MAIN_HWND.store(0, Ordering::Release);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Create the window and tray icon. Must be called on the thread that will
/// run `message_loop`.
pub fn init(start_hidden: bool) -> Result<(), String> {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let hinst = GetModuleHandleW(null());
        let icon = icon::create();
        ICON.store(icon as isize, Ordering::Release);

        let class_name = wide("LeagueAutoMuteWindow");
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: icon,
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: (COLOR_WINDOW + 1) as usize as HBRUSH,
            lpszMenuName: null(),
            lpszClassName: class_name.as_ptr(),
        };
        if RegisterClassW(&wc) == 0 {
            return Err("RegisterClassW failed".into());
        }

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            wide(APP_TITLE).as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            720,
            420,
            null_mut(),
            null_mut(),
            hinst,
            null(),
        );
        if hwnd.is_null() {
            return Err("CreateWindowExW failed".into());
        }
        ShowWindow(hwnd, if start_hidden { SW_HIDE } else { SW_SHOW });
        log(format!("{APP_TITLE} started. Closing this window hides it to the tray."));
        if start_hidden {
            log("started minimised to tray (double-click the tray icon to open)");
        }
    }
    Ok(())
}

pub fn message_loop() -> i32 {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        msg.wParam as i32
    }
}
