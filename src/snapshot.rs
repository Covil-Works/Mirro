use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::adb::AdbClient;

pub fn sanitize_name(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

pub fn get_timestamp_string() -> String {
    #[cfg(windows)]
    unsafe {
        let mut st: windows_sys::Win32::Foundation::SYSTEMTIME = std::mem::zeroed();
        windows_sys::Win32::System::SystemInformation::GetLocalTime(&mut st);
        format!(
            "{:04}-{:02}-{:02}_{:02}-{:02}-{:02}",
            st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond
        )
    }
    #[cfg(not(windows))]
    {
        use std::time::SystemTime;
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("{}", ts)
    }
}

#[derive(Clone, Default)]
pub struct SnapshotManager {
    cache: Arc<RwLock<HashMap<String, slint::SharedPixelBuffer<slint::Rgba8Pixel>>>>,
    ui_weak: Arc<RwLock<Option<slint::Weak<crate::AppWindow>>>>,
}

impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ui_weak: Arc::new(RwLock::new(None)),
        }
    }

    /// Define a referência fraca para a janela principal (AppWindow)
    pub async fn set_ui_weak(&self, ui_weak: slint::Weak<crate::AppWindow>) {
        let mut write = self.ui_weak.write().await;
        *write = Some(ui_weak);
    }

    /// Diretório de cache de snapshots de tela dos aparelhos conectados
    pub fn snapshots_cache_dir() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "Mirro", "Mirro") {
            let dir = proj_dirs.config_dir().join("snapshots");
            let _ = fs::create_dir_all(&dir);
            dir
        } else {
            let dir = PathBuf::from("snapshots");
            let _ = fs::create_dir_all(&dir);
            dir
        }
    }

    /// Diretório onde capturas manuais do usuário são salvas no computador
    pub fn user_screenshots_dir() -> PathBuf {
        crate::config::ConfigManager::get_screenshot_dir()
    }

    /// Caminho do arquivo de cache persistente para um dispositivo específico
    pub fn device_cache_path(serial: &str) -> PathBuf {
        let safe_serial = sanitize_name(serial);
        Self::snapshots_cache_dir().join(format!("{}.png", safe_serial))
    }

    /// Processa os bytes do PNG, redimensiona para 280x570 e aplica desfoque Gaussiano
    async fn process_blur(png_bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
        let png_vec = png_bytes.to_vec();
        tokio::task::spawn_blocking(move || -> Option<(u32, u32, Vec<u8>)> {
            let img = match image::load_from_memory_with_format(&png_vec, image::ImageFormat::Png) {
                Ok(i) => i,
                Err(e) => {
                    warn!("Falha ao decodificar PNG da captura de tela: {:?}", e);
                    return None;
                }
            };

            // Proporção do chassis da moldura (280x570)
            let target_w = 280;
            let target_h = 570;

            let resized = img.resize_exact(target_w, target_h, image::imageops::FilterType::Triangle);
            let blurred = image::imageops::blur(&resized.to_rgba8(), 14.0);

            let (w, h) = (blurred.width(), blurred.height());
            Some((w, h, blurred.into_raw()))
        })
        .await
        .ok()
        .flatten()
    }

    /// Obtém o buffer da imagem desfocada em memória para um dado serial, se disponível
    pub async fn get_cached(&self, serial: &str) -> Option<slint::SharedPixelBuffer<slint::Rgba8Pixel>> {
        let read = self.cache.read().await;
        read.get(serial).cloned()
    }

    /// Carrega o snapshot persistido no disco ou captura via ADB caso ainda não exista.
    /// Não executa nova captura ADB se a imagem já existir no computador.
    pub async fn load_or_capture(
        &self,
        serial: String,
        ui_weak: slint::Weak<crate::AppWindow>,
    ) -> Option<slint::SharedPixelBuffer<slint::Rgba8Pixel>> {
        // 1. Verifica no cache em memória
        if let Some(buf) = self.get_cached(&serial).await {
            let buf_to_send = buf.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_screen_image(slint::Image::from_rgba8(buf_to_send));
                    ui.set_has_screen_image(true);
                }
            });
            return Some(buf);
        }

        // 2. Verifica se já existe snapshot salvo no disco para este dispositivo
        let cache_path = Self::device_cache_path(&serial);
        if cache_path.exists() {
            if let Ok(bytes) = fs::read(&cache_path) {
                if let Some((w, h, raw_bytes)) = Self::process_blur(&bytes).await {
                    let mut pixel_buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(w, h);
                    pixel_buffer.make_mut_bytes().copy_from_slice(&raw_bytes);

                    // Salva em memória
                    {
                        let mut write = self.cache.write().await;
                        write.insert(serial.clone(), pixel_buffer.clone());
                    }

                    // Aplica na UI
                    let buf_to_send = pixel_buffer.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_screen_image(slint::Image::from_rgba8(buf_to_send));
                            ui.set_has_screen_image(true);
                        }
                    });

                    info!(
                        "Snapshot do dispositivo {} restaurado com sucesso do disco: {:?}",
                        serial, cache_path
                    );
                    return Some(pixel_buffer);
                }
            }
        }

        // 3. Se não existe no disco nem em memória, realiza a captura via ADB
        info!("Nenhum snapshot encontrado em disco para {}. Iniciando captura via ADB...", serial);
        let png_bytes = match AdbClient::capture_screenshot(&serial).await {
            Some(bytes) => bytes,
            None => {
                warn!("Não foi possível obter captura de tela do dispositivo {}", serial);
                return None;
            }
        };

        // Salva a captura no computador para reutilização futura sem novo print
        if let Err(e) = fs::write(&cache_path, &png_bytes) {
            warn!("Falha ao salvar snapshot em disco em {:?}: {:?}", cache_path, e);
        } else {
            info!("Snapshot do dispositivo {} salvo com sucesso em disco: {:?}", serial, cache_path);
        }

        // Processa desfoque
        let (w, h, raw_bytes) = match Self::process_blur(&png_bytes).await {
            Some(data) => data,
            None => {
                error!("Erro ao processar desfoque da captura de tela de {}", serial);
                return None;
            }
        };

        let mut pixel_buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(w, h);
        pixel_buffer.make_mut_bytes().copy_from_slice(&raw_bytes);

        // Atualiza cache em memória
        {
            let mut write = self.cache.write().await;
            write.insert(serial.clone(), pixel_buffer.clone());
        }

        // Aplica na UI
        let buf_to_send = pixel_buffer.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_screen_image(slint::Image::from_rgba8(buf_to_send));
                ui.set_has_screen_image(true);
            }
        });

        Some(pixel_buffer)
    }

    /// Realiza a captura de tela solicitada pelo usuário (via botão do app ou companion),
    /// disparando o print nativo no celular (salvo na galeria do dispositivo),
    /// salvando a imagem em alta resolução na pasta configurada no computador,
    /// atualizando o cache persistente do dispositivo e atualizando o mockup na UI.
    pub async fn take_user_screenshot(
        &self,
        serial: &str,
        specific_ui_weak: Option<slint::Weak<crate::AppWindow>>,
    ) -> Option<PathBuf> {
        info!("Executando captura de tela sob demanda do usuário para o dispositivo {}", serial);

        // 1. Obtém a captura de tela para salvar no PC e atualizar o mockup
        // Realizado antes do evento nativo para que a imagem do PC seja limpa (sem animação de flash do celular)
        let png_bytes = match AdbClient::capture_screenshot(serial).await {
            Some(bytes) => bytes,
            None => {
                warn!("Falha ao capturar tela sob demanda para {}", serial);
                return None;
            }
        };

        // 2. Dispara a captura de tela nativa no próprio celular (salva na galeria do celular)
        // O keyevent 120 (KEYCODE_SYSRQ) aciona o serviço nativo de screenshot do Android,
        // gerando animação visual, som de obturador e salvando em DCIM/Screenshots ou Pictures/Screenshots.
        if let Err(e) = AdbClient::trigger_device_screenshot(serial).await {
            warn!("Aviso ao disparar captura nativa no dispositivo {}: {}", serial, e);
        } else {
            info!("Captura nativa disparada com sucesso no dispositivo {}", serial);
        }

        // 3. Salva na pasta de screenshots do usuário no PC
        let user_dir = Self::user_screenshots_dir();
        let timestamp = get_timestamp_string();
        let safe_serial = sanitize_name(serial);
        let user_file_path = user_dir.join(format!("Mirro_Screenshot_{}_{}.png", safe_serial, timestamp));

        if let Err(e) = fs::write(&user_file_path, &png_bytes) {
            error!("Erro ao salvar captura de tela do usuário em {:?}: {:?}", user_file_path, e);
        } else {
            info!("Captura de tela salva com sucesso no computador: {:?}", user_file_path);
        }

        // 4. Atualiza também o cache persistente do dispositivo para o mockup
        let cache_path = Self::device_cache_path(serial);
        let _ = fs::write(&cache_path, &png_bytes);

        // 5. Atualiza o desfoque e a imagem no mockup da UI e exibe notificação
        let ui_weak = match specific_ui_weak {
            Some(w) => Some(w),
            None => self.ui_weak.read().await.clone(),
        };

        if let Some((w, h, raw_bytes)) = Self::process_blur(&png_bytes).await {
            let mut pixel_buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(w, h);
            pixel_buffer.make_mut_bytes().copy_from_slice(&raw_bytes);

            {
                let mut write = self.cache.write().await;
                write.insert(serial.to_string(), pixel_buffer.clone());
            }

            if let Some(ref weak) = ui_weak {
                let buf_to_send = pixel_buffer.clone();
                let file_path_clone = user_file_path.clone();
                let weak_clone = weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = weak_clone.upgrade() {
                        ui.set_screen_image(slint::Image::from_rgba8(buf_to_send));
                        ui.set_has_screen_image(true);
                        ui.set_toast_message(slint::SharedString::from(format!(
                            "Captura salva no celular e no PC:\n{}",
                            file_path_clone.display()
                        )));
                        ui.set_show_toast(true);
                    }
                });

                // Oculta o toast informativo após 4 segundos
                let weak_timer = weak.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(4000)).await;
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = weak_timer.upgrade() {
                            ui.set_show_toast(false);
                        }
                    });
                });
            }
        }

        Some(user_file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("ABC123XYZ"), "ABC123XYZ");
        assert_eq!(sanitize_name("192.168.1.50:5555"), "192_168_1_50_5555");
    }

    #[test]
    fn test_timestamp_format() {
        let ts = get_timestamp_string();
        assert!(!ts.is_empty());
        assert!(ts.contains('_') || ts.len() > 8);
    }

    #[test]
    fn test_directories_creation() {
        let cache_dir = SnapshotManager::snapshots_cache_dir();
        assert!(cache_dir.exists());

        let user_dir = SnapshotManager::user_screenshots_dir();
        assert!(user_dir.exists());
    }
}
