use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::adb::AdbClient;

#[derive(Clone, Default)]
pub struct SnapshotManager {
    cache: Arc<RwLock<HashMap<String, slint::SharedPixelBuffer<slint::Rgba8Pixel>>>>,
}

impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Obtém o buffer da imagem desfocada em cache para um dado serial, se disponível.
    pub async fn get_cached(&self, serial: &str) -> Option<slint::SharedPixelBuffer<slint::Rgba8Pixel>> {
        let read = self.cache.read().await;
        read.get(serial).cloned()
    }

    /// Captura a tela do dispositivo via ADB, aplica redimensionamento e desfoque Gaussiano,
    /// atualiza o cache interno e envia para a UI do Slint de forma assíncrona.
    pub async fn capture_and_apply(
        &self,
        serial: String,
        ui_weak: slint::Weak<crate::AppWindow>,
    ) -> Option<slint::SharedPixelBuffer<slint::Rgba8Pixel>> {
        info!("Iniciando captura de tela com desfoque para dispositivo {}", serial);

        let png_bytes = match AdbClient::capture_screenshot(&serial).await {
            Some(bytes) => bytes,
            None => {
                warn!("Não foi possível obter screenshot do dispositivo {}", serial);
                return None;
            }
        };

        let image_res = tokio::task::spawn_blocking(move || -> Option<(u32, u32, Vec<u8>)> {
            let img = match image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png) {
                Ok(i) => i,
                Err(e) => {
                    warn!("Falha ao decodificar PNG da captura de tela: {:?}", e);
                    return None;
                }
            };

            // Resolução proporcional ao chassis da moldura (280x570)
            let target_w = 280;
            let target_h = 570;

            let resized = img.resize_exact(target_w, target_h, image::imageops::FilterType::Triangle);
            let blurred = image::imageops::blur(&resized.to_rgba8(), 14.0);

            let (w, h) = (blurred.width(), blurred.height());
            Some((w, h, blurred.into_raw()))
        })
        .await;

        let (w, h, raw_bytes) = match image_res {
            Ok(Some(data)) => data,
            Ok(None) => return None,
            Err(e) => {
                error!("Erro na thread de processamento de desfoque: {:?}", e);
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

        info!("Tela capturada, desfocada e aplicada com sucesso para {}", serial);
        Some(pixel_buffer)
    }
}
