slint::include_modules!();

use std::rc::Rc;
use std::sync::Arc;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use tokio::sync::RwLock;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

mod adb;
mod config;
mod mirror;
mod snapshot;
mod tools;

use adb::{AdbClient, DeviceStatus};
use config::ConfigManager;
use mirror::MirrorManager;
use snapshot::SnapshotManager;

fn show_temporary_toast(msg: String, ui_weak: slint::Weak<AppWindow>) {
    let ui_weak_clone = ui_weak.clone();
    let msg_clone = msg.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak_clone.upgrade() {
            ui.set_toast_message(SharedString::from(msg_clone));
            ui.set_show_toast(true);
        }
    });

    let ui_timer = ui_weak.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(4000)).await;
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_timer.upgrade() {
                ui.set_show_toast(false);
            }
        });
    });
}

async fn update_mirro_files(
    serial: Option<String>,
    dir: Option<String>,
    ui_weak: slint::Weak<AppWindow>,
) {
    let serial = match serial {
        Some(s) if !s.is_empty() => s,
        _ => {
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_mirro_files(ModelRc::from(Rc::new(VecModel::from(vec![]))));
                    ui.set_is_in_subfolder(false);
                }
            });
            return;
        }
    };

    let target_dir = match dir {
        Some(d) if !d.is_empty() => d,
        _ => {
            let mut cur = "/sdcard/mirro".to_string();
            if let Some(ui) = ui_weak.upgrade() {
                let uicur = ui.get_current_mirro_dir().to_string();
                if !uicur.is_empty() && uicur.starts_with("/sdcard/mirro") {
                    cur = uicur;
                }
            }
            cur
        }
    };

    let is_sub = target_dir.trim_end_matches('/') != "/sdcard/mirro";

    let files = match AdbClient::list_dir(&serial, &target_dir).await {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!("Falha ao listar arquivos de {}: {}", target_dir, e);
            vec![]
        }
    };

    let file_models: Vec<MirroFileData> = files
        .into_iter()
        .map(|f| MirroFileData {
            name: SharedString::from(f.name),
            file_type: SharedString::from(f.file_type),
            is_dir: f.is_dir,
        })
        .collect();

    let target_dir_clone = target_dir.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let model = Rc::new(VecModel::from(file_models));
            ui.set_mirro_files(ModelRc::from(model));
            ui.set_current_mirro_dir(SharedString::from(target_dir_clone));
            ui.set_is_in_subfolder(is_sub);
        }
    });
}

async fn push_files_to_device(
    serial: String,
    paths: Vec<std::path::PathBuf>,
    target_dir: Option<String>,
    ui_weak: slint::Weak<AppWindow>,
) {
    if paths.is_empty() {
        return;
    }

    let dir = target_dir.unwrap_or_else(|| "/sdcard/mirro".to_string());
    let count = paths.len();
    let first_name = paths[0]
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("arquivo")
        .to_string();

    let msg = if count == 1 {
        format!("Enviando {} para {}...", first_name, dir)
    } else {
        format!("Enviando {} arquivos para {}...", count, dir)
    };

    show_temporary_toast(msg, ui_weak.clone());

    let mut success_count = 0;
    for path in &paths {
        let fname = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("arquivo");
        let remote_dest = format!("{}/{}", dir.trim_end_matches('/'), fname);
        if let Err(e) = AdbClient::push_file_to(&serial, path, &remote_dest).await {
            error!("Erro ao enviar {:?} para {}: {}", path, serial, e);
        } else {
            success_count += 1;
        }
    }

    // Atualiza a lista de arquivos
    update_mirro_files(Some(serial.clone()), Some(dir.clone()), ui_weak.clone()).await;

    // Toast de sucesso ou erro
    let done_msg = if success_count == count {
        if count == 1 {
            format!("{} salvo com sucesso em {}!", first_name, dir)
        } else {
            format!("{} arquivos salvos com sucesso em {}!", count, dir)
        }
    } else {
        format!("{}/{} arquivos enviados com sucesso.", success_count, count)
    };

    show_temporary_toast(done_msg, ui_weak);
}

async fn handle_open_file(
    serial: String,
    dir: String,
    filename: String,
    is_dir: bool,
    ui_weak: slint::Weak<AppWindow>,
) {
    if is_dir {
        let new_dir = format!("{}/{}", dir.trim_end_matches('/'), filename);
        update_mirro_files(Some(serial), Some(new_dir), ui_weak).await;
        return;
    }

    let remote_path = format!("{}/{}", dir.trim_end_matches('/'), filename);
    let cache_dir = std::env::temp_dir().join("mirro_cache");
    let _ = std::fs::create_dir_all(&cache_dir);
    let local_path = cache_dir.join(&filename);

    if let Err(e) = AdbClient::pull_file(&serial, &remote_path, &local_path).await {
        show_temporary_toast(format!("Falha ao baixar arquivo: {}", e), ui_weak);
        return;
    }

    let ext = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let is_img = ["png", "jpg", "jpeg", "webp", "bmp"].contains(&ext.as_str());
    if is_img {
        let local_path_clone = local_path.clone();
        let fname = filename.clone();
        let rpath = remote_path.clone();
        let ui_weak_img = ui_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak_img.upgrade() {
                if let Ok(img) = slint::Image::load_from_path(&local_path_clone) {
                    ui.set_viewer_file_name(SharedString::from(fname));
                    ui.set_viewer_file_path(SharedString::from(rpath));
                    ui.set_viewer_is_image(true);
                    ui.set_viewer_image_data(img);
                    ui.set_viewer_status_badge(SharedString::from("IMAGEM"));
                    ui.set_show_file_viewer(true);
                }
            }
        });
        return;
    }

    let bytes = std::fs::read(&local_path).unwrap_or_default();
    let text_extensions = [
        "txt", "md", "json", "xml", "log", "csv", "sh", "yml", "yaml", "ini",
        "conf", "cfg", "rs", "py", "js", "html", "css", "toml", "env", "properties",
        "sql", "c", "cpp", "h", "java", "kt", "ts", "bat", "cmd", "ps1",
    ];

    let is_likely_text = text_extensions.contains(&ext.as_str())
        || (!bytes.is_empty() && !bytes.contains(&0) && std::str::from_utf8(&bytes).is_ok());

    if is_likely_text {
        let content = String::from_utf8_lossy(&bytes).to_string();
        let fname = filename.clone();
        let rpath = remote_path.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_viewer_file_name(SharedString::from(fname));
                ui.set_viewer_file_path(SharedString::from(rpath));
                ui.set_viewer_is_image(false);
                ui.set_viewer_file_content(SharedString::from(content));
                ui.set_viewer_status_badge(SharedString::from("EDITÁVEL"));
                ui.set_show_file_viewer(true);
            }
        });
    } else {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", "", local_path.to_str().unwrap_or_default()])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn();
        }
        show_temporary_toast(
            "Arquivo aberto no aplicativo padrão do Windows.".to_string(),
            ui_weak,
        );
    }
}

async fn handle_save_viewer_file(
    serial: String,
    remote_path: String,
    content: String,
    ui_weak: slint::Weak<AppWindow>,
) {
    let cache_dir = std::env::temp_dir().join("mirro_cache");
    let _ = std::fs::create_dir_all(&cache_dir);
    let local_path = cache_dir.join("temp_save.tmp");
    if let Err(e) = std::fs::write(&local_path, content.as_bytes()) {
        error!("Erro ao salvar arquivo temporário: {}", e);
        show_temporary_toast(format!("Erro ao preparar arquivo: {}", e), ui_weak);
        return;
    }

    match AdbClient::push_file_to(&serial, &local_path, &remote_path).await {
        Ok(_) => {
            let ui_weak_clone = ui_weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak_clone.upgrade() {
                    ui.set_viewer_status_badge(SharedString::from("SALVO"));
                    ui.set_toast_message(SharedString::from("Arquivo salvo com sucesso no celular!"));
                    ui.set_show_toast(true);
                }
            });
            update_mirro_files(Some(serial), None, ui_weak).await;
        }
        Err(e) => {
            error!("Erro ao enviar arquivo para o celular: {}", e);
            show_temporary_toast(format!("Falha ao salvar no smartphone: {}", e), ui_weak);
        }
    }
}

async fn handle_open_viewer_file_in_pc(
    filename: String,
    ui_weak: slint::Weak<AppWindow>,
) {
    let local_path = std::env::temp_dir().join("mirro_cache").join(&filename);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", local_path.to_str().unwrap_or_default()])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
    show_temporary_toast(
        "Arquivo aberto no aplicativo padrão do Windows.".to_string(),
        ui_weak,
    );
}

async fn handle_paste_file(
    serial: String,
    current_dir: String,
    src_dir: String,
    filename: String,
    is_cut: bool,
    ui_weak: slint::Weak<AppWindow>,
) {
    let src = format!("{}/{}", src_dir.trim_end_matches('/'), filename);
    let dst = if current_dir == src_dir && !is_cut {
        format!("{}/copia_{}", current_dir.trim_end_matches('/'), filename)
    } else {
        format!("{}/{}", current_dir.trim_end_matches('/'), filename)
    };

    let res = if is_cut {
        AdbClient::move_file(&serial, &src, &dst).await
    } else {
        AdbClient::copy_file(&serial, &src, &dst).await
    };

    let ui_weak_clone = ui_weak.clone();
    let msg = match res {
        Ok(_) => {
            if is_cut {
                "Arquivo movido com sucesso!".to_string()
            } else {
                "Arquivo copiado com sucesso!".to_string()
            }
        }
        Err(e) => format!("Falha ao colar arquivo: {}", e),
    };

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak_clone.upgrade() {
            if is_cut {
                ui.set_is_cut_mode(false);
                ui.set_has_clipboard_file(false);
                ui.set_clipboard_file_name(SharedString::from(""));
            }
            ui.set_toast_message(SharedString::from(msg));
            ui.set_show_toast(true);
        }
    });

    update_mirro_files(Some(serial), Some(current_dir), ui_weak).await;
}

async fn handle_delete_file(
    serial: String,
    current_dir: String,
    filename: String,
    ui_weak: slint::Weak<AppWindow>,
) {
    let target = format!("{}/{}", current_dir.trim_end_matches('/'), filename);
    let res = AdbClient::delete_file(&serial, &target).await;
    let ui_weak_clone = ui_weak.clone();
    let msg = match res {
        Ok(_) => format!("{} apagado com sucesso!", filename),
        Err(e) => format!("Falha ao apagar: {}", e),
    };

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak_clone.upgrade() {
            ui.set_selected_file_idx(-1);
            ui.set_selected_file_name(SharedString::from(""));
            ui.set_toast_message(SharedString::from(msg));
            ui.set_show_toast(true);
        }
    });

    update_mirro_files(Some(serial), Some(current_dir), ui_weak).await;
}

async fn handle_download_file(
    serial: String,
    current_dir: String,
    filename: String,
    ui_weak: slint::Weak<AppWindow>,
) {
    let save_path = rfd::FileDialog::new()
        .set_title("Salvar arquivo no computador")
        .set_file_name(&filename)
        .save_file();

    if let Some(path) = save_path {
        let remote_path = format!("{}/{}", current_dir.trim_end_matches('/'), filename);
        let res = AdbClient::pull_file(&serial, &remote_path, &path).await;
        let msg = match res {
            Ok(_) => format!("{} salvo no computador com sucesso!", filename),
            Err(e) => format!("Falha ao salvar no computador: {}", e),
        };
        show_temporary_toast(msg, ui_weak);
    }
}

async fn update_devices(
    mirror_mgr: &MirrorManager,
    snapshot_mgr: &SnapshotManager,
    selected_serial_holder: &Arc<RwLock<Option<String>>>,
    ui_weak: slint::Weak<AppWindow>,
) {
    let devices = AdbClient::list_devices().await;
    let active_serial = mirror_mgr.get_active_serial().await;
    let has_any_ready = devices.iter().any(|d| d.status == DeviceStatus::Ready);
    let first_ready_serial = devices
        .iter()
        .find(|d| d.status == DeviceStatus::Ready)
        .map(|d| d.serial.clone());

    // Assegura que o selected_serial esteja preenchido se houver aparelho conectado
    let current_selected = {
        let mut write = selected_serial_holder.write().await;
        if write.is_none() || !devices.iter().any(|d| write.as_deref() == Some(&d.serial)) {
            *write = first_ready_serial.clone();
        }
        write.clone()
    };

    let device_cards: Vec<DeviceCardData> = devices
        .into_iter()
        .map(|d| {
            let is_mirroring = active_serial.as_deref() == Some(&d.serial);
            let (status_text, is_ready, is_unauthorized) = match d.status {
                DeviceStatus::Ready => {
                    if is_mirroring {
                        ("Espelhando", true, false)
                    } else {
                        ("Conectado", true, false)
                    }
                }
                DeviceStatus::Unauthorized => ("Não Autorizado", false, true),
                DeviceStatus::Offline => ("Offline", false, false),
                DeviceStatus::Other(ref s) => (s.as_str(), false, false),
            };

            DeviceCardData {
                serial: SharedString::from(d.serial),
                model: SharedString::from(d.model),
                display_name: SharedString::from(d.display_name),
                status_text: SharedString::from(status_text),
                status_message: SharedString::from(d.status_message),
                is_ready,
                is_unauthorized,
                is_mirroring,
            }
        })
        .collect();

    let ui_weak_clone = ui_weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak_clone.upgrade() {
            let model = Rc::new(VecModel::from(device_cards));
            ui.set_devices(ModelRc::from(model));
            if !has_any_ready {
                ui.set_has_screen_image(false);
            }
        }
    });

    // Atualiza a lista de arquivos de /sdcard/mirro para o aparelho selecionado
    update_mirro_files(current_selected, None, ui_weak.clone()).await;

    // Se houver aparelho pronto e nenhum screenshot em cache para ele, restaura do disco ou captura
    if let Some(serial) = first_ready_serial {
        if snapshot_mgr.get_cached(&serial).await.is_none() {
            let snapshot_mgr = snapshot_mgr.clone();
            let ui_weak = ui_weak.clone();
            tokio::spawn(async move {
                snapshot_mgr.load_or_capture(serial, ui_weak).await;
            });
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuração de logs estruturados
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    info!("Iniciando o Mirro — Controle & Espelhamento Android...");

    let ui = AppWindow::new()?;

    #[cfg(windows)]
    {
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let title_w: Vec<u16> = "Mirro\0"
                .encode_utf16()
                .collect();
            unsafe {
                let hwnd = windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW(
                    std::ptr::null_mut(),
                    title_w.as_ptr(),
                );
                if !hwnd.is_null() {
                    crate::tools::icon::set_window_icon(hwnd);
                }
            }
        });
    }

    let selected_serial = Arc::new(RwLock::new(None));
    let mirror_manager = MirrorManager::new();
    let snapshot_manager = SnapshotManager::new();
    snapshot_manager.set_ui_weak(ui.as_weak()).await;
    mirror_manager.set_snapshot_manager(snapshot_manager.clone());

    // Inicializa preferências salvas no arquivo de configuração
    let initial_turn_off = ConfigManager::get_turn_screen_off();
    ui.set_turn_screen_off(initial_turn_off);

    let initial_always_on_top = ConfigManager::get_always_on_top();
    ui.set_mirror_always_on_top(initial_always_on_top);

    // Inicializa diretório de salvamento de capturas
    let initial_screenshot_path = ConfigManager::get_screenshot_dir_string();
    ui.set_screenshot_save_path(SharedString::from(initial_screenshot_path));

    // Callback de seleção de pasta para capturas de tela (abre seletor nativo)
    {
        let ui_weak = ui.as_weak();
        ui.on_choose_screenshot_dir(move || {
            let ui_weak = ui_weak.clone();
            std::thread::spawn(move || {
                let current = ConfigManager::get_screenshot_dir();
                let dialog = rfd::FileDialog::new()
                    .set_title("Selecionar pasta para capturas de tela")
                    .set_directory(&current);
                if let Some(folder) = dialog.pick_folder() {
                    let path_str = folder.to_string_lossy().to_string();
                    ConfigManager::set_screenshot_dir(&path_str);
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_screenshot_save_path(SharedString::from(path_str));
                        }
                    });
                }
            });
        });
    }

    // Callback para abrir pasta de capturas no Explorer
    ui.on_open_screenshot_dir(move || {
        let path = ConfigManager::get_screenshot_dir();
        let _ = std::process::Command::new("explorer.exe")
            .arg(&path)
            .spawn();
    });

    // Callback para restaurar pasta padrão de capturas
    {
        let ui_weak = ui.as_weak();
        ui.on_reset_screenshot_dir(move || {
            ConfigManager::reset_screenshot_dir();
            let def_str = ConfigManager::get_screenshot_dir_string();
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_screenshot_save_path(SharedString::from(def_str));
            }
        });
    }

    // Inicializa preferências de atalhos rápidos do widget flutuante
    let initial_sc = ConfigManager::get_quick_shortcuts();
    ui.set_quick_back(initial_sc.back);
    ui.set_quick_recents(initial_sc.recents);
    ui.set_quick_home(initial_sc.home);
    ui.set_quick_power(initial_sc.power);
    ui.set_quick_volup(initial_sc.volup);
    ui.set_quick_voldown(initial_sc.voldown);
    ui.set_quick_screenshot(initial_sc.screenshot);
    ui.set_quick_menu(initial_sc.menu);

    // Callback de alteração da preferência de apagar tela ao espelhar
    ui.on_toggle_turn_screen_off(move |val| {
        ConfigManager::set_turn_screen_off(val);
    });

    // Callback de alteração da preferência de sempre no topo
    ui.on_toggle_always_on_top(move |val| {
        ConfigManager::set_always_on_top(val);
    });

    // Callback de alternância de atalho rápido no painel
    {
        let mirror_mgr = mirror_manager.clone();
        ui.on_toggle_quick_shortcut(move |action, enabled| {
            let act = action.to_string();
            ConfigManager::set_quick_shortcut(&act, enabled);
            mirror_mgr.companion().update_shortcuts(ConfigManager::get_quick_shortcuts());
        });
    }

    // Inicializa suporte a Drag & Drop nativo na janela principal do aplicativo
    {
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        tools::drag_drop::DragDropManager::init(ui.as_weak(), move |paths| {
            let selected_serial = selected_serial_clone.clone();
            let ui_weak = ui_weak.clone();
            tokio::spawn(async move {
                let serial_opt = selected_serial.read().await.clone();
                if let Some(serial) = serial_opt {
                    let dir = if let Some(ui) = ui_weak.upgrade() {
                        Some(ui.get_current_mirro_dir().to_string())
                    } else {
                        None
                    };
                    push_files_to_device(serial, paths, dir, ui_weak).await;
                } else {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_toast_message(SharedString::from(
                                "Conecte um smartphone para transferir arquivos.",
                            ));
                            ui.set_show_toast(true);
                        }
                    });
                }
            });
        });
    }

    // Callback de atualização da lista de arquivos de /sdcard/mirro
    {
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        ui.on_refresh_mirro_files(move || {
            let selected_serial = selected_serial_clone.clone();
            let ui_weak = ui_weak.clone();
            tokio::spawn(async move {
                let serial = selected_serial.read().await.clone();
                update_mirro_files(serial, None, ui_weak).await;
            });
        });
    }

    // Callback para seleção manual de arquivos do computador para envio ao smartphone
    {
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        ui.on_pick_and_send_file(move || {
            let selected_serial = selected_serial_clone.clone();
            let ui_weak = ui_weak.clone();
            tokio::spawn(async move {
                let serial_opt = selected_serial.read().await.clone();
                let serial = match serial_opt {
                    Some(s) => s,
                    None => {
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_toast_message(SharedString::from(
                                    "Conecte um smartphone para transferir arquivos.",
                                ));
                                ui.set_show_toast(true);
                            }
                        });
                        return;
                    }
                };

                let files = rfd::FileDialog::new()
                    .set_title("Selecionar arquivos para enviar ao smartphone (/mirro)")
                    .pick_files();

                if let Some(paths) = files {
                    let dir = if let Some(ui) = ui_weak.upgrade() {
                        Some(ui.get_current_mirro_dir().to_string())
                    } else {
                        None
                    };
                    push_files_to_device(serial, paths, dir, ui_weak).await;
                }
            });
        });
    }

    // Callbacks do Gerenciador de Arquivos (/mirro)
    let last_click_state = Arc::new(tokio::sync::Mutex::new((None::<String>, std::time::Instant::now())));

    // Clique único ou duplo em item da lista
    {
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        let last_click = last_click_state.clone();
        ui.on_file_single_clicked(move |idx, filename, is_dir| {
            let selected_serial = selected_serial_clone.clone();
            let ui_weak = ui_weak.clone();
            let last_click = last_click.clone();
            let filename_str = filename.to_string();
            tokio::spawn(async move {
                let now = std::time::Instant::now();
                let is_double_click = {
                    let mut lock = last_click.lock().await;
                    let (ref last_name, ref last_time) = *lock;
                    let is_match = last_name.as_deref() == Some(&filename_str)
                        && now.duration_since(*last_time) < std::time::Duration::from_millis(400);
                    *lock = (Some(filename_str.clone()), now);
                    is_match
                };

                let serial_opt = selected_serial.read().await.clone();
                let serial = match serial_opt {
                    Some(s) => s,
                    None => return,
                };

                let dir = if let Some(ui) = ui_weak.upgrade() {
                    ui.get_current_mirro_dir().to_string()
                } else {
                    "/sdcard/mirro".to_string()
                };

                if is_double_click {
                    handle_open_file(serial, dir, filename_str, is_dir, ui_weak).await;
                } else {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_selected_file_idx(idx);
                            ui.set_selected_file_name(SharedString::from(filename_str));
                        }
                    });
                }
            });
        });
    }

    // Clique com botão direito em item (Abre menu suspenso igualzinho explorador de arquivos)
    {
        let ui_weak = ui.as_weak();
        ui.on_file_right_clicked(move |idx, filename, is_dir, y_pos| {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_context_menu_file_idx(idx);
                ui.set_context_menu_file_name(filename.clone());
                ui.set_context_menu_is_dir(is_dir);
                ui.set_selected_file_idx(idx);
                ui.set_selected_file_name(filename);
                let clamped_y = y_pos.max(90.0).min(380.0);
                ui.set_context_menu_y(clamped_y);
                ui.set_show_file_context_menu(true);
            }
        });
    }

    // Ação: Abrir Arquivo / Pasta
    {
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        ui.on_open_file(move |filename, is_dir| {
            let selected_serial = selected_serial_clone.clone();
            let ui_weak = ui_weak.clone();
            let fname = filename.to_string();
            tokio::spawn(async move {
                let serial_opt = selected_serial.read().await.clone();
                if let Some(serial) = serial_opt {
                    let dir = if let Some(ui) = ui_weak.upgrade() {
                        ui.get_current_mirro_dir().to_string()
                    } else {
                        "/sdcard/mirro".to_string()
                    };
                    handle_open_file(serial, dir, fname, is_dir, ui_weak).await;
                }
            });
        });
    }

    // Ação: Copiar
    {
        let ui_weak = ui.as_weak();
        ui.on_copy_file_action(move |filename| {
            if let Some(ui) = ui_weak.upgrade() {
                let cur = ui.get_current_mirro_dir();
                ui.set_clipboard_file_name(filename.clone());
                ui.set_clipboard_source_dir(cur);
                ui.set_is_cut_mode(false);
                ui.set_has_clipboard_file(true);
                show_temporary_toast(format!("Copiado: {}", filename), ui_weak.clone());
            }
        });
    }

    // Ação: Recortar
    {
        let ui_weak = ui.as_weak();
        ui.on_cut_file_action(move |filename| {
            if let Some(ui) = ui_weak.upgrade() {
                let cur = ui.get_current_mirro_dir();
                ui.set_clipboard_file_name(filename.clone());
                ui.set_clipboard_source_dir(cur);
                ui.set_is_cut_mode(true);
                ui.set_has_clipboard_file(true);
                show_temporary_toast(format!("Recortado: {}", filename), ui_weak.clone());
            }
        });
    }

    // Ação: Colar
    {
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        ui.on_paste_file_action(move || {
            let selected_serial = selected_serial_clone.clone();
            let ui_weak = ui_weak.clone();
            tokio::spawn(async move {
                let serial_opt = selected_serial.read().await.clone();
                let serial = match serial_opt {
                    Some(s) => s,
                    None => return,
                };

                let (cur_dir, src_dir, filename, is_cut) = if let Some(ui) = ui_weak.upgrade() {
                    (
                        ui.get_current_mirro_dir().to_string(),
                        ui.get_clipboard_source_dir().to_string(),
                        ui.get_clipboard_file_name().to_string(),
                        ui.get_is_cut_mode(),
                    )
                } else {
                    return;
                };

                if filename.is_empty() {
                    return;
                }

                handle_paste_file(serial, cur_dir, src_dir, filename, is_cut, ui_weak).await;
            });
        });
    }

    // Ação: Apagar
    {
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        ui.on_delete_file_action(move |filename| {
            let selected_serial = selected_serial_clone.clone();
            let ui_weak = ui_weak.clone();
            let fname = filename.to_string();
            tokio::spawn(async move {
                let serial_opt = selected_serial.read().await.clone();
                if let Some(serial) = serial_opt {
                    let dir = if let Some(ui) = ui_weak.upgrade() {
                        ui.get_current_mirro_dir().to_string()
                    } else {
                        "/sdcard/mirro".to_string()
                    };
                    handle_delete_file(serial, dir, fname, ui_weak).await;
                }
            });
        });
    }

    // Ação: Baixar para o PC
    {
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        ui.on_download_file_action(move |filename| {
            let selected_serial = selected_serial_clone.clone();
            let ui_weak = ui_weak.clone();
            let fname = filename.to_string();
            tokio::spawn(async move {
                let serial_opt = selected_serial.read().await.clone();
                if let Some(serial) = serial_opt {
                    let dir = if let Some(ui) = ui_weak.upgrade() {
                        ui.get_current_mirro_dir().to_string()
                    } else {
                        "/sdcard/mirro".to_string()
                    };
                    handle_download_file(serial, dir, fname, ui_weak).await;
                }
            });
        });
    }

    // Ação: Voltar para pasta pai (Navegação de subpastas)
    {
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        ui.on_navigate_parent_dir(move || {
            let selected_serial = selected_serial_clone.clone();
            let ui_weak = ui_weak.clone();
            tokio::spawn(async move {
                let serial_opt = selected_serial.read().await.clone();
                if let Some(serial) = serial_opt {
                    let cur = if let Some(ui) = ui_weak.upgrade() {
                        ui.get_current_mirro_dir().to_string()
                    } else {
                        "/sdcard/mirro".to_string()
                    };
                    let clean = cur.trim_end_matches('/');
                    let parent_dir = if clean == "/sdcard/mirro" || !clean.starts_with("/sdcard/mirro") {
                        "/sdcard/mirro".to_string()
                    } else {
                        let parts: Vec<&str> = clean.split('/').collect();
                        if parts.len() > 3 {
                            parts[..parts.len() - 1].join("/")
                        } else {
                            "/sdcard/mirro".to_string()
                        }
                    };
                    update_mirro_files(Some(serial), Some(parent_dir), ui_weak).await;
                }
            });
        });
    }

    // Ação: Salvar arquivo do editor de volta no smartphone
    {
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        ui.on_save_viewer_file(move |content| {
            let selected_serial = selected_serial_clone.clone();
            let ui_weak = ui_weak.clone();
            let cnt = content.to_string();
            tokio::spawn(async move {
                let serial_opt = selected_serial.read().await.clone();
                if let Some(serial) = serial_opt {
                    let rpath = if let Some(ui) = ui_weak.upgrade() {
                        ui.get_viewer_file_path().to_string()
                    } else {
                        String::new()
                    };
                    if !rpath.is_empty() {
                        handle_save_viewer_file(serial, rpath, cnt, ui_weak).await;
                    }
                }
            });
        });
    }

    // Ação: Abrir arquivo do editor no PC
    {
        let ui_weak = ui.as_weak();
        ui.on_open_viewer_file_in_pc(move || {
            let ui_weak_clone = ui_weak.clone();
            let fname = if let Some(ui) = ui_weak.upgrade() {
                ui.get_viewer_file_name().to_string()
            } else {
                String::new()
            };
            if !fname.is_empty() {
                tokio::spawn(async move {
                    handle_open_viewer_file_in_pc(fname, ui_weak_clone).await;
                });
            }
        });
    }

    // Ação: Fechar modal de visualização/edição
    {
        let ui_weak = ui.as_weak();
        ui.on_close_file_viewer(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_show_file_viewer(false);
            }
        });
    }

    // Callback de seleção de dispositivo na barra lateral
    {
        let snapshot_mgr = snapshot_manager.clone();
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        ui.on_device_selected(move |serial| {
            let serial_str = serial.to_string();
            let snapshot_mgr = snapshot_mgr.clone();
            let selected_serial_clone = selected_serial_clone.clone();
            let ui_weak = ui_weak.clone();
            let serial_files = serial_str.clone();
            let ui_weak_files = ui_weak.clone();
            tokio::spawn(async move {
                {
                    let mut write = selected_serial_clone.write().await;
                    *write = Some(serial_str.clone());
                }
                update_mirro_files(Some(serial_files), None, ui_weak_files).await;
                snapshot_mgr.load_or_capture(serial_str, ui_weak).await;
            });
        });
    }

    // Atualização inicial de dispositivos
    {
        let mirror_mgr = mirror_manager.clone();
        let snapshot_mgr = snapshot_manager.clone();
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        tokio::spawn(async move {
            update_devices(&mirror_mgr, &snapshot_mgr, &selected_serial_clone, ui_weak).await;
        });
    }

    // Polling contínuo em segundo plano para detectar conexões/desconexões USB (a cada 2s)
    {
        let mirror_mgr = mirror_manager.clone();
        let snapshot_mgr = snapshot_manager.clone();
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                update_devices(&mirror_mgr, &snapshot_mgr, &selected_serial_clone, ui_weak.clone()).await;
            }
        });
    }

    // Callback de fim de sessão do scrcpy (janela flutuante fechada)
    {
        let mirror_mgr = mirror_manager.clone();
        let snapshot_mgr = snapshot_manager.clone();
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        mirror_manager.set_on_ended(move || {
            let mirror_mgr = mirror_mgr.clone();
            let snapshot_mgr = snapshot_mgr.clone();
            let selected_serial_clone = selected_serial_clone.clone();
            let ui_weak = ui_weak.clone();
            tokio::spawn(async move {
                update_devices(&mirror_mgr, &snapshot_mgr, &selected_serial_clone, ui_weak).await;
            });
        });
    }

    // Callback de atualização manual
    {
        let mirror_mgr = mirror_manager.clone();
        let snapshot_mgr = snapshot_manager.clone();
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        ui.on_refresh_requested(move || {
            let mirror_mgr = mirror_mgr.clone();
            let snapshot_mgr = snapshot_mgr.clone();
            let selected_serial_clone = selected_serial_clone.clone();
            let ui_weak = ui_weak.clone();
            tokio::spawn(async move {
                update_devices(&mirror_mgr, &snapshot_mgr, &selected_serial_clone, ui_weak).await;
            });
        });
    }

    // Callback de alternância de espelhamento (Fast Switching)
    {
        let mirror_mgr = mirror_manager.clone();
        let snapshot_mgr = snapshot_manager.clone();
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        ui.on_mirror_toggled(move |serial, turn_off, always_on_top| {
            let mirror_mgr = mirror_mgr.clone();
            let snapshot_mgr = snapshot_mgr.clone();
            let selected_serial_clone = selected_serial_clone.clone();
            let ui_weak = ui_weak.clone();
            let serial_str = serial.to_string();
            tokio::spawn(async move {
                let is_active = mirror_mgr
                    .get_active_serial()
                    .await
                    .as_deref()
                    == Some(&serial_str);

                if is_active {
                    mirror_mgr.stop_mirroring().await;
                } else {
                    let alias = ConfigManager::get_alias(&serial_str)
                        .unwrap_or_else(|| serial_str.clone());

                    if let Err(e) = mirror_mgr
                        .start_mirroring(serial_str.clone(), alias, turn_off, always_on_top)
                        .await
                    {
                        error!("Falha ao iniciar espelhamento: {}", e);
                    } else {
                        // Ao iniciar espelhamento, assegura que a tela do mockup esteja carregada
                        let snapshot_mgr = snapshot_mgr.clone();
                        let serial_cap = serial_str.clone();
                        let ui_weak_cap = ui_weak.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                            snapshot_mgr.load_or_capture(serial_cap, ui_weak_cap).await;
                        });
                    }
                }
                update_devices(&mirror_mgr, &snapshot_mgr, &selected_serial_clone, ui_weak).await;
            });
        });
    }

    // Callback de ações de navegação (Voltar, Home, Recentes, Ligar/Desligar Tela, Captura de Tela)
    {
        let snapshot_mgr = snapshot_manager.clone();
        let ui_weak = ui.as_weak();
        ui.on_nav_action(move |serial, action| {
            let serial_str = serial.to_string();
            let action_str = action.to_string();
            let snapshot_mgr = snapshot_mgr.clone();
            let ui_weak = ui_weak.clone();
            tokio::spawn(async move {
                if action_str == "screenshot" {
                    let _ = snapshot_mgr.take_user_screenshot(&serial_str, Some(ui_weak)).await;
                    return;
                }
                let keycode = match action_str.as_str() {
                    "back" => 4,
                    "home" => 3,
                    "recents" => 187,
                    "power" => 26,
                    "volup" => 24,
                    "voldown" => 25,
                    "menu" => 82,
                    _ => return,
                };
                let _ = AdbClient::send_keyevent(&serial_str, keycode).await;
            });
        });
    }

    // Callbacks do Modal de Apelido (Renomear)
    {
        let ui_weak = ui.as_weak();
        ui.on_start_rename(move |serial, current_name| {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_rename_serial(serial);
                ui.set_rename_text(current_name);
                ui.set_show_rename_dialog(true);
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        ui.on_cancel_rename(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_show_rename_dialog(false);
            }
        });
    }

    {
        let mirror_mgr = mirror_manager.clone();
        let snapshot_mgr = snapshot_manager.clone();
        let selected_serial_clone = selected_serial.clone();
        let ui_weak = ui.as_weak();
        ui.on_confirm_rename(move |serial, new_name| {
            let serial_str = serial.to_string();
            let new_name_str = new_name.to_string();
            ConfigManager::set_alias(&serial_str, &new_name_str);
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_show_rename_dialog(false);
            }
            let mirror_mgr = mirror_mgr.clone();
            let snapshot_mgr = snapshot_mgr.clone();
            let selected_serial_clone = selected_serial_clone.clone();
            let ui_weak = ui_weak.clone();
            tokio::spawn(async move {
                update_devices(&mirror_mgr, &snapshot_mgr, &selected_serial_clone, ui_weak).await;
            });
        });
    }

    ui.show()?;
    ui.run()?;
    Ok(())
}

