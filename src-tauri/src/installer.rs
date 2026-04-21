use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use tauri::{AppHandle, Emitter};

#[derive(Clone, serde::Serialize)]
struct InstallLogEvent {
    line: String,
}

fn emit_log(app: &AppHandle, line: &str) {
    let _ = app.emit("install-log", InstallLogEvent { line: line.to_string() });
}

// ---- Platform-specific node installation directories ----

#[cfg(target_os = "windows")]
fn user_node_dir() -> PathBuf {
    let local_appdata = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join("AppData").join("Local"));
    local_appdata.join("Programs").join("nodejs")
}

#[cfg(target_os = "macos")]
fn user_node_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".local").join("node")
}

pub fn nodejs_dir() -> Result<PathBuf, String> {
    Ok(user_node_dir())
}

// ---- Find node/npm binaries ----

pub fn find_nodejs_bin() -> Option<(PathBuf, PathBuf)> {
    // 1. 检查系统 PATH 中的 node（当前进程环境变量已包含则优先）
    if Command::new("node").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
        if Command::new("npm").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
            return Some((PathBuf::from("node"), PathBuf::from("npm")));
        }
    }

    // 2. 检查用户目录安装位置
    let node_dir = user_node_dir();
    #[cfg(target_os = "windows")]
    {
        let node = node_dir.join("node.exe");
        let npm = node_dir.join("npm.cmd");
        if node.exists() && npm.exists() {
            return Some((node, npm));
        }
    }
    #[cfg(target_os = "macos")]
    {
        let bin_dir = node_dir.join("bin");
        let node = bin_dir.join("node");
        let npm = bin_dir.join("npm");
        if node.exists() && npm.exists() {
            return Some((node, npm));
        }
    }

    None
}

// ---- npm execution wrapper (handles Windows .cmd limitation) ----

fn npm_cli_js_path() -> Option<PathBuf> {
    let node_dir = user_node_dir();
    #[cfg(target_os = "windows")]
    {
        let win = node_dir.join("node_modules").join("npm").join("bin").join("npm-cli.js");
        if win.exists() {
            return Some(win);
        }
    }
    #[cfg(target_os = "macos")]
    {
        let unix = node_dir.join("lib").join("node_modules").join("npm").join("bin").join("npm-cli.js");
        if unix.exists() {
            return Some(unix);
        }
    }
    None
}

pub fn npm_command(node_path: &Path, npm_path: &Path, args: &[&str]) -> Command {
    let path_str = node_path.to_string_lossy();
    let is_local = path_str.contains(r"Programs\nodejs") || path_str.contains(".local/node");
    if is_local {
        if let Some(npm_cli) = npm_cli_js_path() {
            let mut cmd = Command::new(node_path);
            cmd.arg(&npm_cli);
            cmd.args(args);
            return cmd;
        }
    }
    let mut cmd = Command::new(npm_path);
    cmd.args(args);
    cmd
}

// ---- PATH configuration ----

#[cfg(target_os = "windows")]
fn add_to_user_path(dir: &Path) -> Result<(), String> {
    let dir_str = dir.to_string_lossy().to_string();
    let output = Command::new("powershell")
        .args(&[
            "-Command",
            &format!(
                "$current = [Environment]::GetEnvironmentVariable('Path', 'User'); \
                 if (-not $current.Contains('{}')) {{ \
                     [Environment]::SetEnvironmentVariable('Path', $current + ';{}', 'User') \
                 }}",
                dir_str, dir_str
            ),
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!("添加 PATH 失败: {}", String::from_utf8_lossy(&output.stderr)));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn add_to_user_path(bin_dir: &Path) -> Result<(), String> {
    let home = dirs::home_dir().ok_or("无法获取主目录")?;
    let shell = std::env::var("SHELL").unwrap_or_default();
    let profile = if shell.contains("zsh") || !home.join(".bashrc").exists() {
        home.join(".zshrc")
    } else {
        home.join(".bashrc")
    };

    let path_line = format!(r#"export PATH="{}:$PATH""#, bin_dir.to_string_lossy());

    let content = if profile.exists() {
        fs::read_to_string(&profile).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    if content.contains(&path_line) {
        return Ok(());
    }

    let new_content = format!("{}\n{}\n", content.trim_end(), path_line);
    fs::write(&profile, new_content).map_err(|e| e.to_string())?;
    Ok(())
}

fn update_current_process_path() {
    let current = std::env::var("PATH").unwrap_or_default();
    #[cfg(target_os = "windows")]
    {
        let user_node = user_node_dir();
        std::env::set_var("PATH", format!("{};{}", current, user_node.to_string_lossy()));
    }
    #[cfg(target_os = "macos")]
    {
        let bin_dir = user_node_dir().join("bin");
        std::env::set_var("PATH", format!("{}:{}", current, bin_dir.to_string_lossy()));
    }
}

// ---- Download & install Node.js ----

fn download_and_install_nodejs(app: &AppHandle) -> Result<(), String> {
    emit_log(app, "[!] Node.js 环境未检测到，正在准备自动安装...");

    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let download_dir = home.join(".cc-toolbox").join("downloads");
    fs::create_dir_all(&download_dir).map_err(|e| e.to_string())?;

    let version = "v20.18.3";

    #[cfg(target_os = "windows")]
    {
        let node_dir = user_node_dir();
        fs::create_dir_all(&node_dir).map_err(|e| e.to_string())?;

        let url = format!("https://nodejs.org/dist/{0}/node-{0}-win-x64.zip", version);
        let zip_path = download_dir.join("nodejs.zip");
        let extracted_folder = format!("node-{}-win-x64", version);

        emit_log(app, &format!("[↓] 正在下载 Node.js {}...", version));

        let status = Command::new("curl")
            .args(&["-L", "-o", zip_path.to_str().unwrap(), &url])
            .status()
            .map_err(|e| format!("下载失败: {}", e))?;

        if !status.success() {
            return Err("Node.js 下载失败，请检查网络连接".into());
        }

        emit_log(app, "[📦] 正在安装 Node.js...");

        let status = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    zip_path.display(),
                    node_dir.display()
                ),
            ])
            .status()
            .map_err(|e| format!("解压失败: {}", e))?;

        if !status.success() {
            return Err("解压失败".into());
        }

        // 把子目录内容提到 node_dir 根目录
        let subdir = node_dir.join(&extracted_folder);
        if subdir.exists() {
            for entry in fs::read_dir(&subdir).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let from = entry.path();
                let to = node_dir.join(from.file_name().unwrap());
                if to.exists() {
                    if from.is_dir() {
                        fs::remove_dir_all(&to).map_err(|e| e.to_string())?;
                    } else {
                        fs::remove_file(&to).map_err(|e| e.to_string())?;
                    }
                }
                fs::rename(&from, &to).map_err(|e| e.to_string())?;
            }
            fs::remove_dir(&subdir).map_err(|e| e.to_string())?;
        }

        add_to_user_path(&node_dir)?;
        update_current_process_path();
    }

    #[cfg(target_os = "macos")]
    {
        let node_dir = user_node_dir();
        fs::create_dir_all(&node_dir).map_err(|e| e.to_string())?;

        let arch = if cfg!(target_arch = "aarch64") { "darwin-arm64" } else { "darwin-x64" };
        let url = format!("https://nodejs.org/dist/{0}/node-{0}-{1}.tar.gz", version, arch);
        let tar_path = download_dir.join("nodejs.tar.gz");

        emit_log(app, &format!("[↓] 正在下载 Node.js {}...", version));

        let status = Command::new("curl")
            .args(&["-L", "-o", tar_path.to_str().unwrap(), &url])
            .status()
            .map_err(|e| format!("下载失败: {}", e))?;

        if !status.success() {
            return Err("Node.js 下载失败，请检查网络连接".into());
        }

        emit_log(app, "[📦] 正在安装 Node.js...");

        let status = Command::new("tar")
            .args(&[
                "-xzf",
                tar_path.to_str().unwrap(),
                "-C",
                node_dir.to_str().unwrap(),
                "--strip-components=1",
            ])
            .status()
            .map_err(|e| format!("解压失败: {}", e))?;

        if !status.success() {
            return Err("解压失败".into());
        }

        add_to_user_path(&node_dir.join("bin"))?;
    }

    // 验证安装
    let (node_path, npm_path) = find_nodejs_bin().ok_or("Node.js 安装后验证失败")?;
    let node_ver = Command::new(&node_path).arg("--version").output()
        .map_err(|e| format!("验证失败: {}", e))?;
    let npm_ver = npm_command(&node_path, &npm_path, &["--version"])
        .output()
        .map_err(|e| format!("验证失败: {}", e))?;

    if node_ver.status.success() && npm_ver.status.success() {
        let v = String::from_utf8_lossy(&node_ver.stdout).trim().to_string();
        emit_log(app, &format!("[✓] Node.js 安装完成 ({})", v));
    } else {
        return Err("Node.js 安装验证失败".into());
    }

    Ok(())
}

// ---- Install Claude Code ----

pub fn install_claude_code(app: AppHandle, mirror: String) -> Result<(), String> {
    // 1. 确保 node/npm 可用
    let (node_path, npm_path) = match find_nodejs_bin() {
        Some(paths) => paths,
        None => {
            download_and_install_nodejs(&app)?;
            find_nodejs_bin().ok_or("Node.js 安装后仍不可用")?
        }
    };

    emit_log(&app, &format!("[→] 使用 Node.js: {}", node_path.display()));

    // 2. 配置 npm registry
    let registry_arg = match mirror.as_str() {
        "taobao" => "--registry=https://registry.npmmirror.com",
        _ => "",
    };

    // 3. 安装 claude-code
    emit_log(&app, "[↓] 正在安装 @anthropic-ai/claude-code...");

    let mut cmd = npm_command(&node_path, &npm_path, &["install", "-g", "@anthropic-ai/claude-code"]);

    if !registry_arg.is_empty() {
        cmd.arg(registry_arg);
    }

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("启动 npm 失败: {}", e))?;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let app_clone = app.clone();
        std::thread::spawn(move || {
            for line in reader.lines() {
                if let Ok(line) = line {
                    let _ = app_clone.emit("install-log", InstallLogEvent { line });
                }
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let app_clone = app.clone();
        std::thread::spawn(move || {
            for line in reader.lines() {
                if let Ok(line) = line {
                    let _ = app_clone.emit("install-log", InstallLogEvent { line: format!("[stderr] {}", line) });
                }
            }
        });
    }

    let status = child.wait().map_err(|e| format!("等待 npm 失败: {}", e))?;
    if !status.success() {
        return Err(format!("npm install 退出码: {:?}", status.code()));
    }

    emit_log(&app, "[✓] Claude Code 安装完成！");
    emit_log(&app, "[ℹ] 提示: 已自动配置环境变量，请重启已打开的终端窗口使命令生效。");

    Ok(())
}
