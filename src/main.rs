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

async fn update_mirro_files(serial: Option<String>, ui_weak: slint::Weak<AppWindow>) {
    let serial = match serial {
        Some(s) if !s.is_empty() => s,
        _ => {
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_mirro_files(ModelRc::from(Rc::new(VecModel::from(vec![]))));
                }
            });
            return;
        }
    };

    let files = match AdbClient::list_mirro_files(&serial).await {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!("Falha ao listar arquivos de /sdcard/mirro: {}", e);
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

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let model = Rc::new(VecModel::from(file_models));
            ui.set_mirro_files(ModelRc::from(model));
        }
    });
}

async fn push_files_to_device(
    serial: String,
    paths: Vec<std::path::PathBuf>,
    ui_weak: slint::Weak<AppWindow>,
) {
    if paths.is_empty() {
        return;
    }

    let count = paths.len();
    let first_name = paths[0]
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("arquivo")
        .to_string();

    let msg = if count == 1 {
        format!("Enviando {} para /mirro...", first_name)
    } else {
        format!("Enviando {} arquivos para /mirro...", count)
    };

    let ui_weak_clone = ui_weak.clone();
    let msg_clone = msg.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak_clone.upgrade() {
            ui.set_toast_message(SharedString::from(msg_clone));
            ui.set_show_toast(true);
        }
    });

    let mut success_count = 0;
    for path in &paths {
        if let Err(e) = AdbClient::push_to_mirro(&serial, path).await {
            error!("Erro ao enviar {:?} para {}: {}", path, serial, e);
        } else {
            success_count += 1;
        }
    }

    // Atualiza a lista de arquivos de /sdcard/mirro
    update_mirro_files(Some(serial.clone()), ui_weak.clone()).await;

    // Toast de sucesso ou erro
    let done_msg = if success_count == count {
        if count == 1 {
            format!("{} salvo com sucesso em /mirro!", first_name)
        } else {
            format!("{} arquivos salvos com sucesso em /mirro!", count)
        }
    } else {
        format!("{}/{} arquivos enviados com sucesso.", success_count, count)
    };

    let ui_weak_done = ui_weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak_done.upgrade() {
            ui.set_toast_message(SharedString::from(done_msg));
            ui.set_show_toast(true);
        }
    });

    // Oculta toast após 4 segundos
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
    update_mirro_files(current_selected, ui_weak.clone()).await;

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
                    push_files_to_device(serial, paths, ui_weak).await;
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
                update_mirro_files(serial, ui_weak).await;
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
                    push_files_to_device(serial, paths, ui_weak).await;
                }
            });
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
                update_mirro_files(Some(serial_files), ui_weak_files).await;
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

