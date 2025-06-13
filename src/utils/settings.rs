use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::utils::config::get_config_dir;

#[derive(Deserialize, Serialize)]
pub struct Settings {
    pub limit: u32,
    pub market: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            limit: 5,
            market: "US".to_string(),
        }
    }
}

pub fn load_settings() -> Settings {
    let settings_dir = get_config_dir();
    let settings_path = Path::new(&settings_dir).join("settings.conf");
    
    // Create config directory if it doesn't exist
    if let Some(parent) = settings_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    
    if settings_path.exists() {
        match fs::read_to_string(&settings_path) {
            Ok(content) => {
                toml::from_str(&content).unwrap_or_default()
            }
            Err(_) => {
                let default_settings = Settings::default();
                save_settings(&default_settings);
                default_settings
            }
        }
    } else {
        let default_settings = Settings::default();
        save_settings(&default_settings);
        default_settings
    }
}

pub fn save_settings(settings: &Settings) {
    let settings_dir = get_config_dir();
    let settings_path = Path::new(&settings_dir).join("settings.conf");
    
    if let Ok(content) = toml::to_string(settings) {
        let _ = fs::write(settings_path, content);
    }
}
