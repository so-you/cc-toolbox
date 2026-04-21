use crate::models::AppConfig;
use std::fs;
use std::path::PathBuf;

fn config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("could not determine home directory");
    home.join(".cc-toolbox")
}

fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn ensure_config_dir() -> Result<(), String> {
    let dir = config_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn load_config() -> Result<AppConfig, String> {
    ensure_config_dir()?;
    let path = config_path();
    if !path.exists() {
        let default = AppConfig::default();
        save_config(&default)?;
        return Ok(default);
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    ensure_config_dir()?;
    let path = config_path();
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}
