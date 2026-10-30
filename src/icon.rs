//! Builds the tray/window icon at runtime (a speaker with an X), so no
//! resource compiler is needed.

use windows_sys::Win32::Graphics::Gdi::{CreateBitmap, DeleteObject};
use windows_sys::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, HICON, ICONINFO};

const SIZE: i32 = 32;

fn inside_circle(x: i32, y: i32) -> bool {
    let dx = x - 16;
    let dy = y - 16;
    dx * dx + dy * dy <= 15 * 15
}

fn speaker(x: i32, y: i32) -> bool {
    // Body: small rectangle. Cone: widening triangle to the right.
    let body = (5..=9).contains(&x) && (12..=19).contains(&y);
    let cone = (9..=16).contains(&x) && {
        let half = (x - 9) + 4;
        (16 - half..=15 + half).contains(&y)
    };
    body || cone
}

fn cross(x: i32, y: i32) -> bool {
    // Two diagonals in the box x 19..27, y 12..20, two pixels thick.
    if !((19..=27).contains(&x) && (12..=20).contains(&y)) {
        return false;
    }
    let d1 = (x - 19) - (y - 12);
    let d2 = (x - 19) - (20 - y);
    (0..=1).contains(&d1) || (0..=1).contains(&d2)
}

/// Returns an HICON. Pixels are BGRA, premultiplied is not required here
/// because every pixel is either fully opaque or fully transparent.
pub fn create() -> HICON {
    let mut pixels = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let i = ((y * SIZE + x) * 4) as usize;
            let (b, g, r, a) = if !inside_circle(x, y) {
                (0, 0, 0, 0)
            } else if speaker(x, y) || cross(x, y) {
                (0xFF, 0xFF, 0xFF, 0xFF)
            } else {
                (0x45, 0x45, 0xD6, 0xFF) // muted red
            };
            pixels[i] = b;
            pixels[i + 1] = g;
            pixels[i + 2] = r;
            pixels[i + 3] = a;
        }
    }

    unsafe {
        let color = CreateBitmap(SIZE, SIZE, 1, 32, pixels.as_ptr().cast());
        let mask = CreateBitmap(SIZE, SIZE, 1, 1, std::ptr::null());
        let info = ICONINFO {
            fIcon: 1,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: mask,
            hbmColor: color,
        };
        let icon = CreateIconIndirect(&info);
        DeleteObject(color);
        DeleteObject(mask);
        icon
    }
}
