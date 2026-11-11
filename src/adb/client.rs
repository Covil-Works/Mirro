use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, warn};

use crate::config::ConfigManager;
use crate::tools::locator::Locator;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceStatus {
    Ready,
    Unauthorized,
    Offline,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub serial: String,
    pub model: String,
    pub display_name: String,
    pub status: DeviceStatus,
    pub status_message: String,
}

pub struct AdbClient;

impl AdbClient {
    /// Lista os dispositivos conectados ao computador via `adb devices -l`.
    pub async fn list_devices() -> Vec<DiscoveredDevice> {
        let adb_path = Locator::get_adb_path();
        let mut cmd = Command::new(&adb_path);
        cmd.args(["devices", "-l"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW);

        let output = match cmd.output().await {
            Ok(out) => out,
            Err(e) => {
                error!("Falha ao executar ADB em {:?}: {:?}", adb_path, e);
                return Vec::new();
            }
        };

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            warn!("ADB retornou código de erro: {}", err_msg);
            return Vec::new();
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_devices_output(&stdout)
    }

    fn parse_devices_output(output: &str) -> Vec<DiscoveredDevice> {
        let mut devices = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("List of devices") || line.starts_with('*') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let serial = parts[0].to_string();
            let raw_state = parts.get(1).copied().unwrap_or("unknown");

            let (status, status_message) = match raw_state {
                "device" => (DeviceStatus::Ready, "Pronto para espelhar".to_string()),
                "unauthorized" => (
                    DeviceStatus::Unauthorized,
                    "Celular bloqueado ou não autorizado. Desbloqueie a tela e toque em 'Permitir depuração USB'.".to_string(),
                ),
                "offline" => (
                    DeviceStatus::Offline,
                    "Aparelho offline. Reconecte o cabo USB.".to_string(),
                ),
                other => (
                    DeviceStatus::Other(other.to_string()),
                    format!("Status: {}", other),
                ),
            };

            // Extrai propriedades como model:XYZ product:ABC
            let mut extracted_model = None;
            for part in &parts[2..] {
                if let Some(stripped) = part.strip_prefix("model:") {
                    extracted_model = Some(stripped.replace('_', " "));
                }
            }

            let model = extracted_model.unwrap_or_else(|| serial.clone());
            let display_name = if let Some(alias) = ConfigManager::get_alias(&serial) {
                alias
            } else {
                model.clone()
            };

            devices.push(DiscoveredDevice {
                serial,
                model,
                display_name,
                status,
                status_message,
            });
        }

        devices
    }

    #[cfg(test)]
    pub fn parse_devices_test(output: &str) -> Vec<DiscoveredDevice> {
        Self::parse_devices_output(output)
    }

    pub async fn capture_screenshot(serial: &str) -> Option<Vec<u8>> {
        let adb_path = Locator::get_adb_path();
        let mut cmd = Command::new(&adb_path);
        cmd.args(["-s", serial, "exec-out", "screencap", "-p"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW);

        let output = match cmd.output().await {
            Ok(out) => out,
            Err(e) => {
                error!("Falha ao capturar screenshot via ADB: {:?}", e);
                return None;
            }
        };

        if !output.status.success() || output.stdout.is_empty() {
            return None;
        }

        Some(output.stdout)
    }

    /// Dispara a captura de tela nativa no dispositivo Android via KEYCODE_SYSRQ (120).
    /// Isso aciona o serviço nativo do sistema Android, salvando a captura na galeria do celular
    /// (DCIM/Screenshots ou Pictures/Screenshots) com animação visual e som do obturador.
    pub async fn trigger_device_screenshot(serial: &str) -> Result<(), String> {
        Self::send_keyevent(serial, 120).await
    }

    /// Envia um keyevent Android para o dispositivo (Voltar, Home, Recentes, Power).
    pub async fn send_keyevent(serial: &str, keycode: u32) -> Result<(), String> {
        let adb_path = Locator::get_adb_path();
        let mut cmd = Command::new(&adb_path);
        cmd.args(["-s", serial, "shell", "input", "keyevent", &keycode.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    debug!("Keyevent {} enviado com sucesso para {}", keycode, serial);
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(err)
                }
            }
            Err(e) => Err(format!("Falha ao despachar keyevent: {:?}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_devices_output() {
        let sample = "List of devices attached\n\
                      RF8N1234ABC           device product:r8qxxx model:SM_G780G device:r8q transport_id:1\n\
                      192.168.1.50:5555     unauthorized transport_id:2\n\
                      EMULATOR33X           offline transport_id:3\n";

        let devices = AdbClient::parse_devices_test(sample);
        assert_eq!(devices.len(), 3);

        assert_eq!(devices[0].serial, "RF8N1234ABC");
        assert_eq!(devices[0].model, "SM G780G");
        assert_eq!(devices[0].status, DeviceStatus::Ready);

        assert_eq!(devices[1].serial, "192.168.1.50:5555");
        assert_eq!(devices[1].status, DeviceStatus::Unauthorized);

        assert_eq!(devices[2].serial, "EMULATOR33X");
        assert_eq!(devices[2].status, DeviceStatus::Offline);
    }

    #[tokio::test]
    async fn test_capture_live_device() {
        let devices = AdbClient::list_devices().await;
        if let Some(dev) = devices.first() {
            if dev.status == DeviceStatus::Ready {
                let bytes = AdbClient::capture_screenshot(&dev.serial).await;
                assert!(bytes.is_some(), "Screenshot capture should succeed");
                let data = bytes.unwrap();
                assert!(data.len() > 1000, "Screenshot should have data");
                let img = image::load_from_memory(&data);
                assert!(img.is_ok(), "Screenshot should be valid PNG");
            }
        }
    }
}
