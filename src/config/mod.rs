use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default)]
    pub aliases: HashMap<String, String>,
    #[serde(default = "default_turn_screen_off")]
    pub turn_screen_off: bool,
    #[serde(default = "default_always_on_top")]
    pub always_on_top: bool,
}

fn default_turn_screen_off() -> bool {
    true
}

fn default_always_on_top() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            aliases: HashMap::new(),
            turn_screen_off: true,
            always_on_top: true,
        }
    }
}

pub struct ConfigManager;

impl ConfigManager {
    fn config_path() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "Mirro", "Mirro") {
            let config_dir = proj_dirs.config_dir();
            let _ = fs::create_dir_all(config_dir);
            config_dir.join("config.json")
        } else {
            PathBuf::from("config.json")
        }
    }

    pub fn load() -> AppConfig {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                    return cfg;
                }
            }
        }
        AppConfig::default()
    }

    pub fn save(config: &AppConfig) {
        let path = Self::config_path();
        if let Ok(serialized) = serde_json::to_string_pretty(config) {
            if let Err(e) = fs::write(&path, serialized) {
                error!("Falha ao salvar configuração em {:?}: {:?}", path, e);
            } else {
                info!("Configuração salva com sucesso em {:?}", path);
            }
        }
    }

    pub fn get_alias(serial: &str) -> Option<String> {
        let cfg = Self::load();
        cfg.aliases.get(serial).cloned()
    }

    pub fn set_alias(serial: &str, alias: &str) {
        let mut cfg = Self::load();
        cfg.aliases.insert(serial.to_string(), alias.trim().to_string());
        Self::save(&cfg);
    }

    pub fn get_turn_screen_off() -> bool {
        Self::load().turn_screen_off
    }

    pub fn set_turn_screen_off(val: bool) {
        let mut cfg = Self::load();
        cfg.turn_screen_off = val;
        Self::save(&cfg);
    }

    pub fn get_always_on_top() -> bool {
        Self::load().always_on_top
    }

    pub fn set_always_on_top(val: bool) {
        let mut cfg = Self::load();
        cfg.always_on_top = val;
        Self::save(&cfg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let mut cfg = AppConfig::default();
        cfg.aliases.insert("SERIAL123".to_string(), "Meu Galaxy".to_string());
        let json = serde_json::to_string(&cfg).unwrap();
        let decoded: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.aliases.get("SERIAL123").unwrap(), "Meu Galaxy");
        assert!(decoded.turn_screen_off);
        assert!(decoded.always_on_top);
    }
}
