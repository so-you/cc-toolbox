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

fn check_nodejs() -> VersionStatus {
    if let Some((node_path, _)) = crate::installer::find_nodejs_bin() {
        match Command::new(&node_path).arg("--version").output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout)
                    .lines().next().unwrap_or("").trim().to_string();
                VersionStatus {
                    installed: true,
                    version: Some(version.clone()),
                    required: ">= 18.0.0".into(),
                    message: version,
                }
            }
            _ => VersionStatus {
                installed: false,
                version: None,
                required: ">= 18.0.0".into(),
                message: "未安装".into(),
            },
        }
    } else {
        check_command("node", &["--version"], ">= 18.0.0")
    }
}

fn check_npm() -> VersionStatus {
    if let Some((node_path, npm_path)) = crate::installer::find_nodejs_bin() {
        match crate::installer::npm_command(&node_path, &npm_path, &["--version"]).output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout)
                    .lines().next().unwrap_or("").trim().to_string();
                VersionStatus {
                    installed: true,
                    version: Some(version.clone()),
                    required: ">= 9.0.0".into(),
                    message: version,
                }
            }
            _ => VersionStatus {
                installed: false,
                version: None,
                required: ">= 9.0.0".into(),
                message: "未安装".into(),
            },
        }
    } else {
        check_command("npm", &["--version"], ">= 9.0.0")
    }
}

fn check_claude_code() -> VersionStatus {
    let path_check = check_command("claude", &["--version"], "任意版本");
    if path_check.installed {
        return path_check;
    }
    if let Ok(dir) = crate::installer::nodejs_dir() {
        let claude_bin = if cfg!(target_os = "windows") {
            dir.join("claude.cmd")
        } else {
            dir.join("bin").join("claude")
        };
        if claude_bin.exists() {
            match Command::new(&claude_bin).arg("--version").output() {
                Ok(output) if output.status.success() => {
                    let version = String::from_utf8_lossy(&output.stdout)
                        .lines().next().unwrap_or("").trim().to_string();
                    return VersionStatus {
                        installed: true,
                        version: Some(version.clone()),
                        required: "任意版本".into(),
                        message: version,
                    };
                }
                _ => {}
            }
        }
    }
    path_check
}

#[tauri::command]
pub fn system_check() -> SystemStatus {
    SystemStatus {
        node: Some(check_nodejs()),
        git: Some(check_command("git", &["--version"], "任意版本")),
        npm: Some(check_npm()),
        claude_code: Some(check_claude_code()),
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
pub fn install_claude_code(mirror: String, app: tauri::AppHandle) -> Result<(), String> {
    crate::installer::install_claude_code(app, mirror)
}

#[tauri::command]
pub fn write_aliases(providers: Vec<Provider>) -> Result<String, String> {
    crate::alias_writer::write_aliases(&providers)
}
