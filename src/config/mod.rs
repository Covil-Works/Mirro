use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuickShortcutsConfig {
    #[serde(default = "default_true")]
    pub back: bool,
    #[serde(default = "default_true")]
    pub home: bool,
    #[serde(default = "default_true")]
    pub recents: bool,
    #[serde(default = "default_true")]
    pub power: bool,
    #[serde(default)]
    pub volup: bool,
    #[serde(default)]
    pub voldown: bool,
    #[serde(default)]
    pub screenshot: bool,
    #[serde(default)]
    pub menu: bool,
}

fn default_true() -> bool {
    true
}

impl Default for QuickShortcutsConfig {
    fn default() -> Self {
        Self {
            back: true,
            home: true,
            recents: true,
            power: true,
            volup: false,
            voldown: false,
            screenshot: false,
            menu: false,
        }
    }
}

impl QuickShortcutsConfig {
    pub fn active_count(&self) -> usize {
        let mut count = 0;
        if self.back { count += 1; }
        if self.home { count += 1; }
        if self.recents { count += 1; }
        if self.power { count += 1; }
        if self.volup { count += 1; }
        if self.voldown { count += 1; }
        if self.screenshot { count += 1; }
        if self.menu { count += 1; }
        count
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default)]
    pub aliases: HashMap<String, String>,
    #[serde(default = "default_turn_screen_off")]
    pub turn_screen_off: bool,
    #[serde(default = "default_always_on_top")]
    pub always_on_top: bool,
    #[serde(default)]
    pub quick_shortcuts: QuickShortcutsConfig,
    #[serde(default)]
    pub screenshot_dir: Option<String>,
}

fn default_turn_screen_off() -> bool {
    true
}

fn default_always_on_top() -> bool {
    false
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            aliases: HashMap::new(),
            turn_screen_off: true,
            always_on_top: false,
            quick_shortcuts: QuickShortcutsConfig::default(),
            screenshot_dir: None,
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

    pub fn get_quick_shortcuts() -> QuickShortcutsConfig {
        Self::load().quick_shortcuts
    }

    pub fn set_quick_shortcut(action: &str, enabled: bool) {
        let mut cfg = Self::load();
        match action {
            "back" => cfg.quick_shortcuts.back = enabled,
            "home" => cfg.quick_shortcuts.home = enabled,
            "recents" => cfg.quick_shortcuts.recents = enabled,
            "power" => cfg.quick_shortcuts.power = enabled,
            "volup" => cfg.quick_shortcuts.volup = enabled,
            "voldown" => cfg.quick_shortcuts.voldown = enabled,
            "screenshot" => cfg.quick_shortcuts.screenshot = enabled,
            "menu" => cfg.quick_shortcuts.menu = enabled,
            _ => return,
        }
        Self::save(&cfg);
    }

    pub fn default_screenshot_dir() -> PathBuf {
        let base_dir = directories::UserDirs::new()
            .and_then(|u| u.picture_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| {
                directories::ProjectDirs::from("com", "Mirro", "Mirro")
                    .map(|p| p.data_dir().to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("."))
            });
        base_dir.join("Mirro")
    }

    pub fn get_screenshot_dir() -> PathBuf {
        let cfg = Self::load();
        if let Some(ref custom) = cfg.screenshot_dir {
            let trimmed = custom.trim();
            if !trimmed.is_empty() {
                let path = PathBuf::from(trimmed);
                let _ = fs::create_dir_all(&path);
                return path;
            }
        }
        let default_path = Self::default_screenshot_dir();
        let _ = fs::create_dir_all(&default_path);
        default_path
    }

    pub fn get_screenshot_dir_string() -> String {
        Self::get_screenshot_dir().to_string_lossy().to_string()
    }

    pub fn set_screenshot_dir(path: &str) {
        let mut cfg = Self::load();
        let trimmed = path.trim();
        if trimmed.is_empty() {
            cfg.screenshot_dir = None;
        } else {
            cfg.screenshot_dir = Some(trimmed.to_string());
        }
        Self::save(&cfg);
    }

    pub fn reset_screenshot_dir() {
        let mut cfg = Self::load();
        cfg.screenshot_dir = None;
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
        cfg.quick_shortcuts.screenshot = true;
        cfg.screenshot_dir = Some("C:\\MinhasCapturas".to_string());
        let json = serde_json::to_string(&cfg).unwrap();
        let decoded: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.aliases.get("SERIAL123").unwrap(), "Meu Galaxy");
        assert!(decoded.turn_screen_off);
        assert!(!decoded.always_on_top);
        assert!(decoded.quick_shortcuts.screenshot);
        assert!(decoded.quick_shortcuts.back);
        assert_eq!(decoded.screenshot_dir.as_deref(), Some("C:\\MinhasCapturas"));
    }

    #[test]
    fn test_default_screenshot_dir() {
        let def = ConfigManager::default_screenshot_dir();
        assert!(def.ends_with("Mirro"));
    }
}

