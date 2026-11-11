#[cfg(windows)]
use windows_sys::Win32::Foundation::HWND;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateIconFromResourceEx, IsWindow, SendMessageW, ICON_BIG, ICON_SMALL, WM_SETICON,
};

#[cfg(windows)]
const ICON_BYTES: &[u8] = include_bytes!("../../ui/icons/mirro-icon-256.png");

#[cfg(windows)]
pub fn set_window_icon(hwnd: HWND) {
    if hwnd.is_null() || unsafe { IsWindow(hwnd) } == 0 {
        return;
    }
    unsafe {
        let hicon_big = CreateIconFromResourceEx(
            ICON_BYTES.as_ptr(),
            ICON_BYTES.len() as u32,
            1,          // fIcon = TRUE
            0x00030000, // dwVersion = 3.0
            0,
            0,
            0,
        );
        let hicon_small = CreateIconFromResourceEx(
            ICON_BYTES.as_ptr(),
            ICON_BYTES.len() as u32,
            1,          // fIcon = TRUE
            0x00030000,
            16,
            16,
            0,
        );
        if !hicon_big.is_null() {
            SendMessageW(hwnd, WM_SETICON, ICON_BIG as usize, hicon_big as isize);
        }
        if !hicon_small.is_null() {
            SendMessageW(hwnd, WM_SETICON, ICON_SMALL as usize, hicon_small as isize);
        }
    }
}

#[cfg(not(windows))]
pub fn set_window_icon<T>(_hwnd: T) {}
