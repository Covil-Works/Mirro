slint::include_modules!();

use std::rc::Rc;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

mod adb;
mod config;
mod mirror;
mod tools;

use adb::{AdbClient, DeviceStatus};
use config::ConfigManager;
use mirror::MirrorManager;

async fn update_devices(mirror_mgr: &MirrorManager, ui_weak: slint::Weak<AppWindow>) {
    let devices = AdbClient::list_devices().await;
    let active_serial = mirror_mgr.get_active_serial().await;

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

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let model = Rc::new(VecModel::from(device_cards));
            ui.set_devices(ModelRc::from(model));
        }
    });
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
    let mirror_manager = MirrorManager::new();

    // Inicializa preferências salvas no arquivo de configuração
    let initial_turn_off = ConfigManager::get_turn_screen_off();
    ui.set_turn_screen_off(initial_turn_off);

    // Callback de alteração da preferência de apagar tela ao espelhar
    ui.on_toggle_turn_screen_off(move |val| {
        ConfigManager::set_turn_screen_off(val);
    });

    // Atualização inicial de dispositivos
    {
        let mirror_mgr = mirror_manager.clone();
        let ui_weak = ui.as_weak();
        tokio::spawn(async move {
            update_devices(&mirror_mgr, ui_weak).await;
        });
    }

    // Polling contínuo em segundo plano para detectar conexões/desconexões USB (a cada 2s)
    {
        let mirror_mgr = mirror_manager.clone();
        let ui_weak = ui.as_weak();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                update_devices(&mirror_mgr, ui_weak.clone()).await;
            }
        });
    }

    // Callback de fim de sessão do scrcpy (janela flutuante fechada)
    {
        let mirror_mgr = mirror_manager.clone();
        let ui_weak = ui.as_weak();
        mirror_manager.set_on_ended(move || {
            let mirror_mgr = mirror_mgr.clone();
            let ui_weak = ui_weak.clone();
            tokio::spawn(async move {
                update_devices(&mirror_mgr, ui_weak).await;
            });
        });
    }

    // Callback de atualização manual
    {
        let mirror_mgr = mirror_manager.clone();
        let ui_weak = ui.as_weak();
        ui.on_refresh_requested(move || {
            let mirror_mgr = mirror_mgr.clone();
            let ui_weak = ui_weak.clone();
            tokio::spawn(async move {
                update_devices(&mirror_mgr, ui_weak).await;
            });
        });
    }

    // Callback de alternância de espelhamento (Fast Switching)
    {
        let mirror_mgr = mirror_manager.clone();
        let ui_weak = ui.as_weak();
        ui.on_mirror_toggled(move |serial, turn_off| {
            let mirror_mgr = mirror_mgr.clone();
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
                    if let Err(e) = mirror_mgr.start_mirroring(serial_str.clone(), alias, turn_off).await {
                        error!("Falha ao iniciar espelhamento: {}", e);
                    }
                }
                update_devices(&mirror_mgr, ui_weak).await;
            });
        });
    }

    // Callback de ações de navegação (Voltar, Home, Recentes, Ligar/Desligar Tela)
    {
        ui.on_nav_action(move |serial, action| {
            let serial_str = serial.to_string();
            let action_str = action.to_string();
            tokio::spawn(async move {
                let keycode = match action_str.as_str() {
                    "back" => 4,
                    "home" => 3,
                    "recents" => 187,
                    "power" => 26,
                    "volup" => 24,
                    "voldown" => 25,
                    "screenshot" => 120,
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
        let ui_weak = ui.as_weak();
        ui.on_confirm_rename(move |serial, new_name| {
            let serial_str = serial.to_string();
            let new_name_str = new_name.to_string();
            ConfigManager::set_alias(&serial_str, &new_name_str);
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_show_rename_dialog(false);
            }
            let mirror_mgr = mirror_mgr.clone();
            let ui_weak = ui_weak.clone();
            tokio::spawn(async move {
                update_devices(&mirror_mgr, ui_weak).await;
            });
        });
    }

    ui.run()?;
    Ok(())
}
