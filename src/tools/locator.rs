use std::path::PathBuf;
use tracing::{debug, info};

pub struct Locator;

impl Locator {
    /// Obtém o caminho do binário `adb.exe`.
    pub fn get_adb_path() -> PathBuf {
        if let Some(path) = Self::find_binary("adb.exe") {
            info!("ADB localizado em: {:?}", path);
            return path;
        }

        // Fallback: busca no Android SDK local no AppData
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let sdk_path = PathBuf::from(local_appdata)
                .join("Android")
                .join("Sdk")
                .join("platform-tools")
                .join("adb.exe");
            if sdk_path.exists() {
                info!("ADB localizado no Android SDK: {:?}", sdk_path);
                return sdk_path;
            }
        }

        // Último fallback: assume que está no PATH do sistema
        debug!("Fallback para adb no PATH");
        PathBuf::from("adb.exe")
    }

    /// Obtém o caminho do binário `scrcpy.exe`.
    pub fn get_scrcpy_path() -> PathBuf {
        if let Some(path) = Self::find_binary("scrcpy.exe") {
            info!("scrcpy localizado em: {:?}", path);
            return path;
        }

        debug!("Fallback para scrcpy no PATH");
        PathBuf::from("scrcpy.exe")
    }

    /// Procura um binário nos diretórios de ferramentas empacotadas ou de desenvolvimento.
    fn find_binary(binary_name: &str) -> Option<PathBuf> {
        let mut candidates = Vec::new();

        // 1. Relativo ao diretório do executável em execução (e subindo diretórios)
        if let Ok(current_exe) = std::env::current_exe() {
            let mut curr = current_exe.parent();
            while let Some(dir) = curr {
                candidates.push(dir.join("tools").join("scrcpy").join(binary_name));
                candidates.push(dir.join("tools").join(binary_name));
                candidates.push(dir.join(binary_name));
                curr = dir.parent();
            }
        }

        // 2. Relativo ao diretório de trabalho atual (dev mode e subindo diretórios)
        if let Ok(cwd) = std::env::current_dir() {
            let mut curr = Some(cwd.as_path());
            while let Some(dir) = curr {
                candidates.push(dir.join("tools").join("scrcpy").join(binary_name));
                candidates.push(dir.join("tools").join(binary_name));
                candidates.push(dir.join(binary_name));
                curr = dir.parent();
            }
        }

        for path in candidates {
            if path.exists() {
                return Some(path);
            }
        }

        // 3. Checa se o comando está diretamente no PATH do sistema
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let full = dir.join(binary_name);
                if full.exists() {
                    return Some(full);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locator_finds_tools() {
        let adb = Locator::get_adb_path();
        let scrcpy = Locator::get_scrcpy_path();
        assert!(adb.exists() || adb.to_str().unwrap() == "adb.exe");
        assert!(scrcpy.exists() || scrcpy.to_str().unwrap() == "scrcpy.exe");
    }
}
