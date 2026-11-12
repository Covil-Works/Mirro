use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, info, warn};

use crate::config::ConfigManager;
use crate::tools::locator::Locator;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirroFileItem {
    pub name: String,
    pub is_dir: bool,
    pub file_type: String,
}

impl MirroFileItem {
    pub fn new(raw_name: &str, is_dir: bool) -> Self {
        let name = raw_name.trim_end_matches('/').to_string();
        let file_type = if is_dir {
            "PASTA".to_string()
        } else {
            let ext = std::path::Path::new(&name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("ARQUIVO");
            ext.to_ascii_uppercase()
        };
        Self {
            name,
            is_dir,
            file_type,
        }
    }
}

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

    /// Garante que o diretório /sdcard/mirro exista no smartphone.
    /// Se a pasta já existir, a operação não gera erro e prossegue normalmente.
    pub async fn ensure_mirro_dir(serial: &str) -> Result<(), String> {
        let adb_path = Locator::get_adb_path();
        let mut cmd = Command::new(&adb_path);
        cmd.args(["-s", serial, "shell", "mkdir", "-p", "/sdcard/mirro"])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(err)
                }
            }
            Err(e) => Err(format!("Falha ao criar pasta /sdcard/mirro: {:?}", e)),
        }
    }

    /// Envia um arquivo local para /sdcard/mirro/ no smartphone.
    #[allow(dead_code)]
    pub async fn push_to_mirro(serial: &str, local_path: &std::path::Path) -> Result<(), String> {
        Self::ensure_mirro_dir(serial).await?;

        let adb_path = Locator::get_adb_path();
        let mut cmd = Command::new(&adb_path);
        cmd.args(["-s", serial, "push"])
            .arg(local_path)
            .arg("/sdcard/mirro/")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    info!(
                        "Arquivo {:?} enviado para /sdcard/mirro/ no dispositivo {}",
                        local_path, serial
                    );
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(err)
                }
            }
            Err(e) => Err(format!("Falha ao executar adb push: {:?}", e)),
        }
    }

    #[allow(dead_code)]
    /// Lista os arquivos e pastas existentes em /sdcard/mirro/ no smartphone.
    pub async fn list_mirro_files(serial: &str) -> Result<Vec<MirroFileItem>, String> {
        Self::list_dir(serial, "/sdcard/mirro").await
    }

    /// Garante a existência de qualquer diretório remoto no smartphone.
    pub async fn ensure_dir(serial: &str, dir: &str) -> Result<(), String> {
        let adb_path = Locator::get_adb_path();
        let mut cmd = Command::new(&adb_path);
        cmd.args(["-s", serial, "shell", "mkdir", "-p", dir])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(err)
                }
            }
            Err(e) => Err(format!("Falha ao criar diretório remoto {}: {:?}", dir, e)),
        }
    }

    /// Lista os arquivos e pastas existentes em qualquer diretório no smartphone.
    pub async fn list_dir(serial: &str, dir: &str) -> Result<Vec<MirroFileItem>, String> {
        let clean_dir = dir.trim_end_matches('/');
        let _ = Self::ensure_dir(serial, clean_dir).await;

        let target_dir = format!("{}/", clean_dir);
        let adb_path = Locator::get_adb_path();
        let mut cmd = Command::new(&adb_path);
        cmd.args(["-s", serial, "shell", "ls", "-1p", &target_dir])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW);

        match cmd.output().await {
            Ok(output) => {
                if !output.status.success() {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    return Err(err);
                }
                let stdout = String::from_utf8_lossy(&output.stdout);
                Ok(Self::parse_mirro_files(&stdout))
            }
            Err(e) => Err(format!("Falha ao listar arquivos de {}: {:?}", target_dir, e)),
        }
    }

    /// Exclui arquivo ou pasta no smartphone (rm -rf).
    pub async fn delete_file(serial: &str, remote_path: &str) -> Result<(), String> {
        let adb_path = Locator::get_adb_path();
        let mut cmd = Command::new(&adb_path);
        cmd.args(["-s", serial, "shell", "rm", "-rf", remote_path])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    info!("Arquivo {} apagado com sucesso em {}", remote_path, serial);
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(err)
                }
            }
            Err(e) => Err(format!("Falha ao apagar arquivo {}: {:?}", remote_path, e)),
        }
    }

    /// Copia arquivo ou pasta remotamente no smartphone (cp -r).
    pub async fn copy_file(serial: &str, src: &str, dst: &str) -> Result<(), String> {
        let adb_path = Locator::get_adb_path();
        let mut cmd = Command::new(&adb_path);
        cmd.args(["-s", serial, "shell", "cp", "-r", src, dst])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    info!("Copiado {} -> {} em {}", src, dst, serial);
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(err)
                }
            }
            Err(e) => Err(format!("Falha ao copiar de {} para {}: {:?}", src, dst, e)),
        }
    }

    /// Move ou recorta arquivo ou pasta remotamente no smartphone (mv).
    pub async fn move_file(serial: &str, src: &str, dst: &str) -> Result<(), String> {
        let adb_path = Locator::get_adb_path();
        let mut cmd = Command::new(&adb_path);
        cmd.args(["-s", serial, "shell", "mv", src, dst])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    info!("Movido {} -> {} em {}", src, dst, serial);
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(err)
                }
            }
            Err(e) => Err(format!("Falha ao mover de {} para {}: {:?}", src, dst, e)),
        }
    }

    /// Baixa um arquivo do smartphone para o computador local (adb pull).
    pub async fn pull_file(
        serial: &str,
        remote_path: &str,
        local_path: &std::path::Path,
    ) -> Result<(), String> {
        let adb_path = Locator::get_adb_path();
        let mut cmd = Command::new(&adb_path);
        cmd.args([
            "-s",
            serial,
            "pull",
            remote_path,
            local_path.to_str().unwrap_or_default(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    info!("Arquivo {} baixado para {:?}", remote_path, local_path);
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(err)
                }
            }
            Err(e) => Err(format!("Falha ao executar adb pull de {}: {:?}", remote_path, e)),
        }
    }

    /// Envia um arquivo local para um caminho remoto específico no smartphone (adb push).
    pub async fn push_file_to(
        serial: &str,
        local_path: &std::path::Path,
        remote_path: &str,
    ) -> Result<(), String> {
        let adb_path = Locator::get_adb_path();
        let mut cmd = Command::new(&adb_path);
        cmd.args([
            "-s",
            serial,
            "push",
            local_path.to_str().unwrap_or_default(),
            remote_path,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    info!(
                        "Arquivo {:?} enviado para {} em {}",
                        local_path, remote_path, serial
                    );
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(err)
                }
            }
            Err(e) => Err(format!("Falha ao executar adb push para {}: {:?}", remote_path, e)),
        }
    }

    pub fn parse_mirro_files(output: &str) -> Vec<MirroFileItem> {
        let mut items = Vec::new();
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("ls:") {
                continue;
            }
            let is_dir = trimmed.ends_with('/');
            items.push(MirroFileItem::new(trimmed, is_dir));
        }

        // Ordena: pastas primeiro, depois arquivos por ordem alfabética (case-insensitive)
        items.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        items
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

    #[test]
    fn test_parse_mirro_files() {
        let output = "fotos/\nnotas.txt\ndocumento.pdf\nbackup/\naplicativo.apk\n";
        let items = AdbClient::parse_mirro_files(output);
        assert_eq!(items.len(), 5);

        // Pastas primeiro
        assert!(items[0].is_dir);
        assert_eq!(items[0].name, "backup");
        assert_eq!(items[0].file_type, "PASTA");

        assert!(items[1].is_dir);
        assert_eq!(items[1].name, "fotos");
        assert_eq!(items[1].file_type, "PASTA");

        // Arquivos depois
        assert!(!items[2].is_dir);
        assert_eq!(items[2].name, "aplicativo.apk");
        assert_eq!(items[2].file_type, "APK");

        assert!(!items[3].is_dir);
        assert_eq!(items[3].name, "documento.pdf");
        assert_eq!(items[3].file_type, "PDF");

        assert!(!items[4].is_dir);
        assert_eq!(items[4].name, "notas.txt");
        assert_eq!(items[4].file_type, "TXT");
    }

    #[test]
    fn test_file_item_extension_types() {
        let file_txt = MirroFileItem::new("documento.txt", false);
        assert_eq!(file_txt.file_type, "TXT");
        assert!(!file_txt.is_dir);

        let file_png = MirroFileItem::new("foto.png", false);
        assert_eq!(file_png.file_type, "PNG");

        let folder = MirroFileItem::new("minha_pasta/", true);
        assert_eq!(folder.name, "minha_pasta");
        assert_eq!(folder.file_type, "PASTA");
        assert!(folder.is_dir);
    }
}
