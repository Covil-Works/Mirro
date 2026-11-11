use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};
use slint::ComponentHandle;

use crate::adb::AdbClient;
use crate::config::{ConfigManager, QuickShortcutsConfig};
use crate::mirror::manager::MirrorManager;
use crate::CompanionWindow;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{HWND, RECT};
#[cfg(windows)]
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetSystemMetrics, GetWindowLongW, GetWindowRect, IsIconic, IsWindow,
    SetWindowLongW, SetWindowPos, ShowWindow, GWL_EXSTYLE, HWND_TOPMOST, SM_CXSCREEN,
    SWP_NOACTIVATE, SW_HIDE, SW_SHOWNOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
};

#[cfg(windows)]
#[repr(C)]
struct MARGINS {
    cx_left_width: i32,
    cx_right_width: i32,
    cy_top_height: i32,
    cy_bottom_height: i32,
}

#[cfg(windows)]
#[link(name = "dwmapi")]
extern "system" {
    fn DwmExtendFrameIntoClientArea(hWnd: HWND, pMarInset: *const MARGINS) -> i32;
}

#[cfg(windows)]
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
fn get_dpi_scale(hwnd: HWND) -> f32 {
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    if dpi > 0 {
        dpi as f32 / 96.0
    } else {
        1.0
    }
}

const LOGICAL_WIDTH: f32 = 52.0;
const LOGICAL_COLLAPSED_HEIGHT: f32 = 52.0;
const BASE_EXPANDED_HEIGHT: f32 = 56.0;
const BUTTON_STEP_HEIGHT: f32 = 39.0;

#[derive(Clone)]
pub struct CompanionManager {
    stop_signal: Arc<std::sync::Mutex<Option<Arc<AtomicBool>>>>,
    active_window: Arc<std::sync::Mutex<Option<slint::Weak<CompanionWindow>>>>,
    active_count: Arc<AtomicIsize>,
}

impl CompanionManager {
    pub fn new() -> Self {
        Self {
            stop_signal: Arc::new(std::sync::Mutex::new(None)),
            active_window: Arc::new(std::sync::Mutex::new(None)),
            active_count: Arc::new(AtomicIsize::new(5)),
        }
    }

    pub fn stop(&self) {
        if let Ok(mut lock) = self.stop_signal.lock() {
            if let Some(sig) = lock.take() {
                sig.store(true, Ordering::SeqCst);
            }
        }
        if let Ok(mut lock) = self.active_window.lock() {
            *lock = None;
        }
    }

    pub fn update_shortcuts(&self, shortcuts: QuickShortcutsConfig) {
        let count = (shortcuts.active_count() + 1) as isize;
        self.active_count.store(count, Ordering::SeqCst);

        if let Ok(lock) = self.active_window.lock() {
            if let Some(weak) = lock.as_ref() {
                let weak_clone = weak.clone();
                let sc = shortcuts;
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(w) = weak_clone.upgrade() {
                        w.set_show_back(sc.back);
                        w.set_show_home(sc.home);
                        w.set_show_recents(sc.recents);
                        w.set_show_power(sc.power);
                        w.set_show_volup(sc.volup);
                        w.set_show_voldown(sc.voldown);
                        w.set_show_screenshot(sc.screenshot);
                        w.set_show_menu(sc.menu);
                    }
                });
            }
        }
    }

    #[cfg(windows)]
    pub fn start(
        &self,
        serial: String,
        display_name: String,
        mirror_mgr: MirrorManager,
    ) {
        self.stop();

        let stop_signal = Arc::new(AtomicBool::new(false));
        if let Ok(mut lock) = self.stop_signal.lock() {
            *lock = Some(stop_signal.clone());
        }

        let initial_sc = ConfigManager::get_quick_shortcuts();
        self.active_count.store((initial_sc.active_count() + 1) as isize, Ordering::SeqCst);

        let stop_signal_clone = stop_signal.clone();
        let active_win_holder = self.active_window.clone();
        let active_count_holder = self.active_count.clone();

        std::thread::spawn(move || {
            let scrcpy_title = format!("Mirro - {}", display_name);
            let scrcpy_title_w = to_wide(&scrcpy_title);

            let mut scrcpy_hwnd: HWND = std::ptr::null_mut();
            for _ in 0..120 {
                if stop_signal_clone.load(Ordering::SeqCst) {
                    return;
                }
                unsafe {
                    let hwnd = FindWindowW(std::ptr::null_mut(), scrcpy_title_w.as_ptr());
                    if !hwnd.is_null() && IsWindow(hwnd) != 0 {
                        scrcpy_hwnd = hwnd;
                        break;
                    }
                }
                std::thread::sleep(Duration::from_millis(50));
            }

            if scrcpy_hwnd.is_null() {
                warn!("Janela do scrcpy ({}) não localizada para anexar companion", scrcpy_title);
                return;
            }

            info!("Janela do scrcpy detectada. Inicializando companion widget anti-aliased...");

            let is_expanded_atomic = Arc::new(AtomicBool::new(false));
            let comp_hwnd_atomic = Arc::new(AtomicIsize::new(0));
            let comp_window_weak = Arc::new(std::sync::Mutex::new(None::<slint::Weak<CompanionWindow>>));

            let is_exp_slint = is_expanded_atomic.clone();
            let comp_hwnd_slint = comp_hwnd_atomic.clone();
            let weak_holder = comp_window_weak.clone();
            let serial_clone = serial.clone();
            let mm_clone = mirror_mgr.clone();
            let count_atomic = active_count_holder.clone();

            let (tx_ready, rx_ready) = std::sync::mpsc::channel();

            let sc_init = initial_sc.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Ok(comp_win) = CompanionWindow::new() {
                    comp_win.set_device_serial(slint::SharedString::from(serial_clone.clone()));
                    comp_win.set_is_expanded(false);

                    // Inicializa opções de atalhos rápidos
                    comp_win.set_show_back(sc_init.back);
                    comp_win.set_show_home(sc_init.home);
                    comp_win.set_show_recents(sc_init.recents);
                    comp_win.set_show_power(sc_init.power);
                    comp_win.set_show_volup(sc_init.volup);
                    comp_win.set_show_voldown(sc_init.voldown);
                    comp_win.set_show_screenshot(sc_init.screenshot);
                    comp_win.set_show_menu(sc_init.menu);

                    let win_weak = comp_win.as_weak();
                    let is_exp = is_exp_slint.clone();
                    let hwnd_atom = comp_hwnd_slint.clone();
                    let count_for_toggle = count_atomic.clone();

                    comp_win.on_toggle_expanded(move || {
                        if let Some(w) = win_weak.upgrade() {
                            let next = !w.get_is_expanded();
                            w.set_is_expanded(next);
                            is_exp.store(next, Ordering::SeqCst);

                            let hwnd_val = hwnd_atom.load(Ordering::SeqCst);
                            if hwnd_val != 0 {
                                let hwnd = hwnd_val as HWND;
                                let scale = get_dpi_scale(hwnd);
                                let w_phys = (LOGICAL_WIDTH * scale).round() as i32;

                                let cnt = count_for_toggle.load(Ordering::SeqCst) as f32;
                                let exp_h = BASE_EXPANDED_HEIGHT + (cnt * BUTTON_STEP_HEIGHT);
                                let h_phys = ((if next { exp_h } else { LOGICAL_COLLAPSED_HEIGHT }) * scale).round() as i32;

                                unsafe {
                                    let mut current_rect: RECT = std::mem::zeroed();
                                    GetWindowRect(hwnd, &mut current_rect);
                                    SetWindowPos(
                                        hwnd,
                                        HWND_TOPMOST,
                                        current_rect.left,
                                        current_rect.top,
                                        w_phys,
                                        h_phys,
                                        SWP_NOACTIVATE,
                                    );
                                }
                            }
                        }
                    });

                    let s = serial_clone.clone();
                    let mm = mm_clone.clone();
                    comp_win.on_action_triggered(move |action| {
                        let act = action.to_string();
                        let s_dev = s.clone();
                        let mm_inner = mm.clone();
                        tokio::spawn(async move {
                            match act.as_str() {
                                "back" => {
                                    let _ = AdbClient::send_keyevent(&s_dev, 4).await;
                                }
                                "home" => {
                                    let _ = AdbClient::send_keyevent(&s_dev, 3).await;
                                }
                                "recents" => {
                                    let _ = AdbClient::send_keyevent(&s_dev, 187).await;
                                }
                                "power" => {
                                    let _ = AdbClient::send_keyevent(&s_dev, 26).await;
                                }
                                "volup" => {
                                    let _ = AdbClient::send_keyevent(&s_dev, 24).await;
                                }
                                "voldown" => {
                                    let _ = AdbClient::send_keyevent(&s_dev, 25).await;
                                }
                                "screenshot" => {
                                    let _ = AdbClient::send_keyevent(&s_dev, 120).await;
                                }
                                "menu" => {
                                    let _ = AdbClient::send_keyevent(&s_dev, 82).await;
                                }
                                "close" => {
                                    mm_inner.stop_mirroring().await;
                                }
                                _ => {}
                            }
                        });
                    });

                    let _ = comp_win.show();
                    *weak_holder.lock().unwrap() = Some(comp_win.as_weak());
                    let _ = tx_ready.send(comp_win.as_weak());
                }
            });

            if let Ok(comp_weak) = rx_ready.recv_timeout(Duration::from_secs(2)) {
                if let Ok(mut lock) = active_win_holder.lock() {
                    *lock = Some(comp_weak);
                }
            }

            let comp_title_w = to_wide("Mirro Companion");
            let mut comp_hwnd: HWND = std::ptr::null_mut();
            for _ in 0..60 {
                if stop_signal_clone.load(Ordering::SeqCst) {
                    return;
                }
                unsafe {
                    let hwnd = FindWindowW(std::ptr::null_mut(), comp_title_w.as_ptr());
                    if !hwnd.is_null() && IsWindow(hwnd) != 0 {
                        comp_hwnd = hwnd;
                        break;
                    }
                }
                std::thread::sleep(Duration::from_millis(30));
            }

            if comp_hwnd.is_null() {
                warn!("Não foi possível obter o HWND do companion widget");
                return;
            }

            comp_hwnd_atomic.store(comp_hwnd as isize, Ordering::SeqCst);

            unsafe {
                // Remove do switcher de tarefas e mantém sempre no topo
                let ex_style = GetWindowLongW(comp_hwnd, GWL_EXSTYLE);
                SetWindowLongW(
                    comp_hwnd,
                    GWL_EXSTYLE,
                    ex_style | (WS_EX_TOOLWINDOW as i32) | (WS_EX_TOPMOST as i32),
                );

                // DWM Composition Transparency: permite anti-aliasing suave por GPU sem recorte 1-bit
                let margins = MARGINS {
                    cx_left_width: -1,
                    cx_right_width: -1,
                    cy_top_height: -1,
                    cy_bottom_height: -1,
                };
                let _ = DwmExtendFrameIntoClientArea(comp_hwnd, &margins);
            }

            let mut was_visible = true;
            let mut last_x = -9999;
            let mut last_y = -9999;
            let mut last_w = -9999;
            let mut last_h = -9999;

            loop {
                if stop_signal_clone.load(Ordering::SeqCst) {
                    break;
                }

                unsafe {
                    if IsWindow(scrcpy_hwnd) == 0 {
                        info!("Janela do scrcpy finalizada. Fechando companion.");
                        break;
                    }

                    let is_minimized = IsIconic(scrcpy_hwnd) != 0;
                    if is_minimized {
                        if was_visible {
                            ShowWindow(comp_hwnd, SW_HIDE);
                            was_visible = false;
                        }
                        std::thread::sleep(Duration::from_millis(100));
                        continue;
                    } else if !was_visible {
                        ShowWindow(comp_hwnd, SW_SHOWNOACTIVATE);
                        was_visible = true;
                    }

                    let mut scrcpy_rect: RECT = std::mem::zeroed();
                    GetWindowRect(scrcpy_hwnd, &mut scrcpy_rect);

                    let is_exp = is_expanded_atomic.load(Ordering::SeqCst);
                    let scale = get_dpi_scale(comp_hwnd);

                    let cnt = active_count_holder.load(Ordering::SeqCst) as f32;
                    let exp_h = BASE_EXPANDED_HEIGHT + (cnt * BUTTON_STEP_HEIGHT);

                    let target_w = (LOGICAL_WIDTH * scale).round() as i32;
                    let target_h = ((if is_exp { exp_h } else { LOGICAL_COLLAPSED_HEIGHT }) * scale).round() as i32;
                    let gap = (10.0 * scale).round() as i32;

                    let screen_w = GetSystemMetrics(SM_CXSCREEN);
                    let mut target_x = scrcpy_rect.right + gap;

                    // Se ultrapassar a margem da tela à direita, ancora no canto esquerdo
                    if target_x + target_w > screen_w {
                        target_x = (scrcpy_rect.left - target_w - gap).max(0);
                    }
                    let target_y = scrcpy_rect.top + (8.0 * scale).round() as i32;

                    if target_x != last_x || target_y != last_y || target_w != last_w || target_h != last_h {
                        SetWindowPos(
                            comp_hwnd,
                            HWND_TOPMOST,
                            target_x,
                            target_y,
                            target_w,
                            target_h,
                            SWP_NOACTIVATE,
                        );

                        last_x = target_x;
                        last_y = target_y;
                        last_w = target_w;
                        last_h = target_h;
                    }
                }

                std::thread::sleep(Duration::from_millis(16));
            }

            let weak_to_hide = comp_window_weak.lock().unwrap().take();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(weak) = weak_to_hide {
                    if let Some(win) = weak.upgrade() {
                        let _ = win.hide();
                    }
                }
            });
        });
    }

    #[cfg(not(windows))]
    pub fn start(
        &self,
        _serial: String,
        _display_name: String,
        _mirror_mgr: MirrorManager,
    ) {}
}
