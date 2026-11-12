use std::path::PathBuf;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{error, info};

#[cfg(windows)]
use windows_sys::Win32::Foundation::HWND;
#[cfg(windows)]
use windows_sys::Win32::System::Ole::RevokeDragDrop;
#[cfg(windows)]
use windows_sys::Win32::UI::Shell::{DragAcceptFiles, DragFinish, DragQueryFileW, HDROP};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DefWindowProcW, FindWindowW, SetWindowLongPtrW,
    GWLP_WNDPROC, WM_DROPFILES, WNDPROC,
};

pub type DropCallback = Box<dyn Fn(Vec<PathBuf>) + Send + Sync + 'static>;

static PREV_WNDPROC: AtomicIsize = AtomicIsize::new(0);
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

        info!("Drag & Drop detectado na janela: {} arquivo(s)", paths.len());

        let callback_opt = {
            let lock = DROP_CALLBACK.lock().unwrap();
            lock.clone()
        };

        if let Some(callback) = callback_opt {
            callback(paths);
        }

        return 0;
    }

    let prev = PREV_WNDPROC.load(Ordering::SeqCst);
    if prev != 0 {
        let wndproc_fn: WNDPROC = std::mem::transmute(prev);
        CallWindowProcW(wndproc_fn, hwnd, msg, wparam, lparam)
    } else {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

pub struct DragDropManager;

impl DragDropManager {
    #[cfg(windows)]
    pub fn init<F>(callback: F)
    where
        F: Fn(Vec<PathBuf>) + Send + Sync + 'static,
    {
        {
            let mut lock = DROP_CALLBACK.lock().unwrap();
            *lock = Some(Arc::new(Box::new(callback)));
        }

        std::thread::spawn(|| {
            let title_w: Vec<u16> = "Mirro\0".encode_utf16().collect();
            let mut found_hwnd: HWND = std::ptr::null_mut();

            for _ in 0..60 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                unsafe {
                    let hwnd = FindWindowW(std::ptr::null_mut(), title_w.as_ptr());
                    if !hwnd.is_null() {
                        found_hwnd = hwnd;
                        break;
                    }
                }
            }

            if found_hwnd.is_null() {
                error!("Janela principal do Mirro não encontrada para registrar Drag & Drop.");
                return;
            }

            unsafe {
                // Revoga qualquer OLE IDropTarget anterior para que DragAcceptFiles receba WM_DROPFILES
                RevokeDragDrop(found_hwnd);
                DragAcceptFiles(found_hwnd, 1);

                let prev = SetWindowLongPtrW(
                    found_hwnd,
                    GWLP_WNDPROC,
                    drag_drop_wndproc as *const () as usize as isize,
                );

                PREV_WNDPROC.store(prev, Ordering::SeqCst);
                info!("Drag & Drop nativo registrado com sucesso no HWND {:?}", found_hwnd);
            }
        });
    }

    #[cfg(not(windows))]
    pub fn init<F>(_callback: F)
    where
        F: Fn(Vec<PathBuf>) + Send + Sync + 'static,
    {
    }
}
