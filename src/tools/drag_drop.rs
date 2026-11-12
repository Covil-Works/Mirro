use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{error, info};

use crate::AppWindow;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{BOOL, HWND};
#[cfg(windows)]
use windows_sys::Win32::System::Ole::RevokeDragDrop;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::GetCurrentProcessId;
#[cfg(windows)]
use windows_sys::Win32::UI::Shell::{DragAcceptFiles, DragFinish, DragQueryFileW, HDROP};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, ChangeWindowMessageFilterEx, DefWindowProcW, EnumChildWindows,
    EnumWindows, GetWindowTextW, GetWindowThreadProcessId, SetWindowLongPtrW,
    GWLP_WNDPROC, WM_DROPFILES, WNDPROC,
};

pub type DropCallback = Box<dyn Fn(Vec<PathBuf>) + Send + Sync + 'static>;

static WINDOW_PROCS: Mutex<Option<HashMap<isize, isize>>> = Mutex::new(None);
static DROP_CALLBACK: Mutex<Option<Arc<DropCallback>>> = Mutex::new(None);

#[cfg(windows)]
unsafe extern "system" fn drag_drop_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    if msg == WM_DROPFILES {
        let hdrop = wparam as HDROP;
        let count = DragQueryFileW(hdrop, 0xFFFFFFFF, std::ptr::null_mut(), 0);
        let mut paths = Vec::new();

        for i in 0..count {
            let len = DragQueryFileW(hdrop, i, std::ptr::null_mut(), 0);
            if len > 0 {
                let mut buf = vec![0u16; (len + 1) as usize];
                DragQueryFileW(hdrop, i, buf.as_mut_ptr(), (len + 1) as u32);
                buf.pop(); // remove trailing null
                let path_str = String::from_utf16_lossy(&buf);
                paths.push(PathBuf::from(path_str));
            }
        }

        DragFinish(hdrop);

        info!("Drag & Drop (WM_DROPFILES) capturado com sucesso: {} arquivo(s)", paths.len());

        let callback_opt = {
            let lock = DROP_CALLBACK.lock().unwrap();
            lock.clone()
        };

        if let Some(callback) = callback_opt {
            callback(paths);
        }

        return 0;
    }

    let prev = {
        let lock = WINDOW_PROCS.lock().unwrap();
        lock.as_ref().and_then(|m| m.get(&(hwnd as isize)).copied()).unwrap_or(0)
    };

    if prev != 0 {
        let wndproc_fn: WNDPROC = std::mem::transmute(prev);
        CallWindowProcW(wndproc_fn, hwnd, msg, wparam, lparam)
    } else {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

#[cfg(windows)]
unsafe fn subclass_window(hwnd: HWND) {
    let key = hwnd as isize;
    let prev = SetWindowLongPtrW(
        hwnd,
        GWLP_WNDPROC,
        drag_drop_wndproc as *const () as usize as isize,
    );
    if prev != 0 && prev != drag_drop_wndproc as *const () as usize as isize {
        let mut map = WINDOW_PROCS.lock().unwrap();
        if map.is_none() {
            *map = Some(HashMap::new());
        }
        map.as_mut().unwrap().insert(key, prev);
        info!("Subclassing de Drag & Drop aplicada no HWND {:?}", hwnd);
    }
}

#[cfg(windows)]
pub fn find_main_window() -> HWND {
    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: isize) -> BOOL {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == GetCurrentProcessId() {
            let mut title_buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512);
            if len > 0 {
                let title = String::from_utf16_lossy(&title_buf[..len as usize]);
                if title == "Mirro" || (title.starts_with("Mirro") && !title.contains("Companion")) {
                    let out_ptr = lparam as *mut HWND;
                    *out_ptr = hwnd;
                    return 0; // Para a busca, encontramos a janela principal
                }
            }
        }
        1
    }

    let mut found: HWND = std::ptr::null_mut();
    unsafe {
        EnumWindows(Some(enum_proc), &mut found as *mut HWND as isize);
    }
    found
}

#[cfg(windows)]
fn setup_drag_drop_on_gui_thread(hwnd: HWND) {
    unsafe {
        // 1. Revoga o OLE IDropTarget na thread dona da janela (thread da UI)
        let hr = RevokeDragDrop(hwnd);
        info!("RevokeDragDrop no HWND principal {:?} retornou: 0x{:08X}", hwnd, hr as u32);

        // 2. Permite mensagens através do filtro UIPI (mesmo com integridades diferentes)
        ChangeWindowMessageFilterEx(hwnd, WM_DROPFILES, 1, std::ptr::null_mut());
        ChangeWindowMessageFilterEx(hwnd, 0x0049, 1, std::ptr::null_mut());
        ChangeWindowMessageFilterEx(hwnd, 0x004A, 1, std::ptr::null_mut());

        // 3. Ativa DragAcceptFiles e aplica subclassing no HWND principal
        DragAcceptFiles(hwnd, 1);
        subclass_window(hwnd);

        // 4. Também aplica em qualquer janela filha que o winit/Slint possa ter criado
        unsafe extern "system" fn enum_child(child: HWND, _lparam: isize) -> BOOL {
            let hr_c = RevokeDragDrop(child);
            info!("RevokeDragDrop em janela filha {:?} retornou: 0x{:08X}", child, hr_c as u32);
            ChangeWindowMessageFilterEx(child, WM_DROPFILES, 1, std::ptr::null_mut());
            ChangeWindowMessageFilterEx(child, 0x0049, 1, std::ptr::null_mut());
            ChangeWindowMessageFilterEx(child, 0x004A, 1, std::ptr::null_mut());
            DragAcceptFiles(child, 1);
            subclass_window(child);
            1
        }

        EnumChildWindows(hwnd, Some(enum_child), 0);
        info!("Drag & Drop nativo configurado com sucesso para a janela Mirro!");
    }
}

pub struct DragDropManager;

impl DragDropManager {
    #[cfg(windows)]
    pub fn init<F>(_ui_weak: slint::Weak<AppWindow>, callback: F)
    where
        F: Fn(Vec<PathBuf>) + Send + Sync + 'static,
    {
        {
            let mut lock = DROP_CALLBACK.lock().unwrap();
            *lock = Some(Arc::new(Box::new(callback)));
        }

        tokio::spawn(async move {
            let mut found_hwnd: usize = 0;

            for _ in 0..100 {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                let hwnd = find_main_window();
                if !hwnd.is_null() {
                    found_hwnd = hwnd as usize;
                    break;
                }
            }

            if found_hwnd == 0 {
                error!("Janela principal do Mirro não encontrada para registrar Drag & Drop.");
                return;
            }

            // Executa obrigatoriamente na thread de UI do Slint para respeitar o apartment COM
            let _ = slint::invoke_from_event_loop(move || {
                setup_drag_drop_on_gui_thread(found_hwnd as HWND);
            });
        });
    }

    #[cfg(not(windows))]
    pub fn init<F>(_ui_weak: slint::Weak<AppWindow>, _callback: F)
    where
        F: Fn(Vec<PathBuf>) + Send + Sync + 'static,
    {
    }
}
