use crate::models::{SystemStatus, VersionStatus, Provider};
use std::process::Command;

fn check_command(cmd: &str, arg: &[&str], required: &str) -> VersionStatus {
    match Command::new(cmd).args(arg).output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let installed = !version.is_empty();
            VersionStatus {
                installed,
                version: if installed { Some(version.clone()) } else { None },
                required: required.into(),
                message: if installed { version } else { "未安装".into() },
            }
        }
        _ => VersionStatus {
            installed: false,
            version: None,
            required: required.into(),
            message: "未安装".into(),
        },
    }
}

#[tauri::command]
pub fn system_check() -> SystemStatus {
    SystemStatus {
        node: Some(check_command("node", &["--version"], ">= 18.0.0")),
        git: Some(check_command("git", &["--version"], "任意版本")),
        npm: Some(check_command("npm", &["--version"], ">= 9.0.0")),
        claude_code: Some(check_command("claude", &["--version"], "任意版本")),
    }
}

#[tauri::command]
pub fn load_providers() -> Result<Vec<Provider>, String> {
    let config = crate::config_store::load_config()?;
    Ok(config.providers)
}

#[tauri::command]
pub fn save_provider(provider: Provider) -> Result<(), String> {
    let mut config = crate::config_store::load_config()?;
    if let Some(existing) = config.providers.iter_mut().find(|p| p.id == provider.id) {
        *existing = provider;
    } else {
        config.providers.push(provider);
    }
    crate::config_store::save_config(&config)
}

#[tauri::command]
pub async fn install_claude_code(mirror: String, app: tauri::AppHandle) -> Result<(), String> {
    crate::installer::install_claude_code(app, mirror)
}

#[tauri::command]
pub fn write_aliases(providers: Vec<Provider>) -> Result<String, String> {
    crate::alias_writer::write_aliases(&providers)
}
