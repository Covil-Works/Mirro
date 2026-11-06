#[cfg(windows)]
pub fn embed_scrcpy_in_frame(host_title: &str, child_title: &str) {
    use std::ptr::null_mut;
    use std::thread::sleep;
    use std::time::Duration;
    use tracing::info;
    use windows_sys::Win32::Foundation::{BOOL, HWND};
    use windows_sys::Win32::Graphics::Gdi::{CreateRoundRectRgn, HRGN};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FindWindowW, GetWindowLongW, SetParent, SetWindowLongW, SetWindowPos,
        GWL_STYLE, HWND_TOP, SWP_FRAMECHANGED, SWP_SHOWWINDOW, WS_CHILD,
        WS_CLIPCHILDREN, WS_POPUP, WS_CAPTION, WS_THICKFRAME,
    };

    extern "system" {
        fn SetWindowRgn(hWnd: HWND, hRgn: HRGN, bRedraw: BOOL) -> i32;
    }

    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    let host_title_w = to_wide(host_title);
    let child_title_w = to_wide(child_title);

    std::thread::spawn(move || {
        // Passo 1: Localiza a janela do Mirro Floating Frame e corta suas bordas no formato arredondado do celular
        for _ in 0..120 {
            sleep(Duration::from_millis(50));
            unsafe {
                let host_hwnd = FindWindowW(null_mut(), host_title_w.as_ptr());
                if !host_hwnd.is_null() {
                    // Recorta o chassi do celular com cantos arredondados (310x630, raio 48px)
                    let host_rgn = CreateRoundRectRgn(0, 0, 310, 630, 48, 48);
                    if !host_rgn.is_null() {
                        let _ = SetWindowRgn(host_hwnd, host_rgn, 1);
                    }
                    break;
                }
            }
        }

        // Passo 2: Localiza a janela do scrcpy, embute e recorta a tela interna nos cantos
        for _ in 0..120 {
            sleep(Duration::from_millis(50));
            unsafe {
                let host_hwnd = FindWindowW(null_mut(), host_title_w.as_ptr());
                let child_hwnd = FindWindowW(null_mut(), child_title_w.as_ptr());

                if !host_hwnd.is_null() && !child_hwnd.is_null() {
                    info!("Incorporando janela do scrcpy com recorte arredondado dentro da moldura...");
                    let host_style = GetWindowLongW(host_hwnd, GWL_STYLE);
                    SetWindowLongW(host_hwnd, GWL_STYLE, host_style | (WS_CLIPCHILDREN as i32));

                    let mut child_style = GetWindowLongW(child_hwnd, GWL_STYLE);
                    child_style &= !(WS_POPUP as i32 | WS_CAPTION as i32 | WS_THICKFRAME as i32);
                    child_style |= WS_CHILD as i32;
                    SetWindowLongW(child_hwnd, GWL_STYLE, child_style);

                    SetParent(child_hwnd, host_hwnd);

                    // Posiciona a tela no interior da moldura (x: 5, y: 30, w: 300, h: 584)
                    SetWindowPos(
                        child_hwnd,
                        HWND_TOP,
                        5,
                        30,
                        300,
                        584,
                        SWP_SHOWWINDOW | SWP_FRAMECHANGED,
                    );

                    // Recorta a tela interna do scrcpy acompanhando o raio dos cantos
                    let child_rgn = CreateRoundRectRgn(0, 0, 300, 584, 34, 34);
                    if !child_rgn.is_null() {
                        let _ = SetWindowRgn(child_hwnd, child_rgn, 1);
                    }
                    break;
                }
            }
        }
    });
}

#[cfg(not(windows))]
pub fn embed_scrcpy_in_frame(_host_title: &str, _child_title: &str) {}

#[cfg(windows)]
pub fn handle_drag_window(host_title: &str) {
    use std::ptr::null_mut;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FindWindowW, SendMessageW, HTCAPTION, WM_NCLBUTTONDOWN,
    };

    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    let title_w = to_wide(host_title);
    unsafe {
        let hwnd = FindWindowW(null_mut(), title_w.as_ptr());
        if !hwnd.is_null() {
            ReleaseCapture();
            SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as usize, 0);
        }
    }
}

#[cfg(not(windows))]
pub fn handle_drag_window(_host_title: &str) {}
