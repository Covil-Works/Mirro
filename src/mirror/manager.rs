use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, warn};

use crate::tools::locator::Locator;

pub type OnSessionEnded = Box<dyn Fn() + Send + Sync + 'static>;

#[derive(Clone)]
pub struct MirrorManager {
    active_child: Arc<Mutex<Option<Child>>>,
    active_serial: Arc<RwLock<Option<String>>>,
    on_ended: Arc<Mutex<Option<OnSessionEnded>>>,
}

impl MirrorManager {
    pub fn new() -> Self {
        Self {
            active_child: Arc::new(Mutex::new(None)),
            active_serial: Arc::new(RwLock::new(None)),
            on_ended: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_on_ended<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        if let Ok(mut lock) = self.on_ended.try_lock() {
            *lock = Some(Box::new(callback));
        }
    }

    /// Retorna o número de série do aparelho atualmente espelhado, se houver.
    pub async fn get_active_serial(&self) -> Option<String> {
        let read = self.active_serial.read().await;
        read.clone()
    }

    /// Inicia o espelhamento via scrcpy.
    /// Se outro aparelho estiver espelhado, encerra-o de imediato (Fast Switching).
    pub async fn start_mirroring(
        &self,
        serial: String,
        display_name: String,
        turn_screen_off: bool,
    ) -> Result<(), String> {
        // Encerra sessão anterior caso ativa
        self.stop_mirroring().await;

        let scrcpy_path = Locator::get_scrcpy_path();
        let adb_path = Locator::get_adb_path();
        let scrcpy_dir = scrcpy_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| Path::new(".").to_path_buf());
        let server_path = scrcpy_dir.join("scrcpy-server");

        info!(
            "Iniciando espelhamento para {} ({}) usando {:?}",
            display_name, serial, scrcpy_path
        );

        let window_title = format!("Mirro - {}", display_name);

        let mut cmd = Command::new(&scrcpy_path);
        cmd.current_dir(&scrcpy_dir);

        // Garante que o scrcpy use exatamente o mesmo binário ADB e servidor empacotado
        cmd.env("ADB", &adb_path);
        if server_path.exists() {
            cmd.env("SCRCPY_SERVER_PATH", &server_path);
        }

        // Argumentos de execução
        let mut args = vec![
            "-s".to_string(),
            serial.clone(),
            "--always-on-top".to_string(),
            "--stay-awake".to_string(), // Evita desconexão USB por suspensão de energia
            "--window-title".to_string(),
            window_title,
        ];

        if turn_screen_off {
            args.push("--turn-screen-off".to_string());
        }

        cmd.args(&args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let err_msg = format!("Erro ao iniciar scrcpy: {:?}", e);
                error!("{}", err_msg);
                return Err(err_msg);
            }
        };

        // Redireciona logs de stdout e stderr para o console do Mirro
        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    info!("[scrcpy stdout] {}", line);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    warn!("[scrcpy stderr] {}", line);
                }
            });
        }

        {
            let mut serial_lock = self.active_serial.write().await;
            *serial_lock = Some(serial.clone());
        }

        {
            let mut child_lock = self.active_child.lock().await;
            *child_lock = Some(child);
        }

        // Tarefa em segundo plano para monitorar o encerramento do processo
        let active_serial_clone = self.active_serial.clone();
        let active_child_clone = self.active_child.clone();
        let on_ended_clone = self.on_ended.clone();
        let monitored_serial = serial.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

                let mut finished = false;
                {
                    let mut lock = active_child_clone.lock().await;
                    if let Some(child) = lock.as_mut() {
                        match child.try_wait() {
                            Ok(Some(status)) => {
                                info!(
                                    "scrcpy para {} encerrou com status: {:?}",
                                    monitored_serial, status
                                );
                                finished = true;
                            }
                            Ok(None) => {
                                // Ainda em execução
                            }
                            Err(e) => {
                                warn!("Falha ao monitorar processo scrcpy: {:?}", e);
                                finished = true;
                            }
                        }
                    } else {
                        break;
                    }

                    if finished {
                        *lock = None;
                    }
                }

                if finished {
                    let mut serial_lock = active_serial_clone.write().await;
                    if serial_lock.as_deref() == Some(&monitored_serial) {
                        *serial_lock = None;
                    }

                    if let Some(callback) = on_ended_clone.lock().await.as_ref() {
                        callback();
                    }
                    break;
                }
            }
        });

        Ok(())
    }

    /// Encerra imediatamente a sessão de espelhamento ativa.
    pub async fn stop_mirroring(&self) {
        let mut child_lock = self.active_child.lock().await;
        if let Some(mut child) = child_lock.take() {
            info!("Encerrando sessão ativa de scrcpy...");
            let _ = child.kill().await;
        }

        let mut serial_lock = self.active_serial.write().await;
        *serial_lock = None;

        if let Some(callback) = self.on_ended.lock().await.as_ref() {
            callback();
        }
    }
}
