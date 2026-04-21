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

fn nodejs_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    Ok(home.join(".cc-toolbox").join("nodejs"))
}

fn find_nodejs_bin() -> Option<(PathBuf, PathBuf)> {
    // 1. 检查本地安装的 node
    if let Ok(dir) = nodejs_dir() {
        if cfg!(target_os = "windows") {
            let node = dir.join("node.exe");
            let npm = dir.join("npm.cmd");
            if node.exists() && npm.exists() {
                return Some((node, npm));
            }
        } else {
            let node = dir.join("bin").join("node");
            let npm = dir.join("bin").join("npm");
            if node.exists() && npm.exists() {
                return Some((node, npm));
            }
        }
    }

    // 2. 检查系统 PATH 中的 node
    if Command::new("node").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
        if Command::new("npm").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
            return Some((PathBuf::from("node"), PathBuf::from("npm")));
        }
    }

    None
}

fn download_and_install_nodejs(app: &AppHandle) -> Result<(), String> {
    emit_log(app, "[!] Node.js 环境未检测到，正在准备自动安装...");

    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let download_dir = home.join(".cc-toolbox").join("downloads");
    let node_dir = nodejs_dir()?;

    fs::create_dir_all(&download_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&node_dir).map_err(|e| e.to_string())?;

    let version = "v20.18.3";

    let (url, filename, extracted_folder) = if cfg!(target_os = "windows") {
        (
            format!("https://nodejs.org/dist/{0}/node-{0}-win-x64.zip", version),
            "nodejs.zip",
            format!("node-{}-win-x64", version),
        )
    } else if cfg!(target_os = "macos") {
        let arch = if cfg!(target_arch = "aarch64") { "darwin-arm64" } else { "darwin-x64" };
        (
            format!("https://nodejs.org/dist/{0}/node-{0}-{1}.tar.gz", version, arch),
            "nodejs.tar.gz",
            format!("node-{}-{}", version, arch),
        )
    } else {
        return Err("不支持的操作系统".into());
    };

    let download_path = download_dir.join(&filename);

    emit_log(app, &format!("[↓] 正在下载 Node.js {}...", version));

    // 下载
    let status = Command::new("curl")
        .args(&["-L", "-o", download_path.to_str().unwrap(), &url])
        .status()
        .map_err(|e| format!("下载失败: {}", e))?;

    if !status.success() {
        return Err("Node.js 下载失败，请检查网络连接".into());
    }

    emit_log(app, "[📦] 正在解压 Node.js...");

    // 解压
    if cfg!(target_os = "windows") {
        let status = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    download_path.display(),
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
    } else {
        let status = Command::new("tar")
            .args(&[
                "-xzf",
                download_path.to_str().unwrap(),
                "-C",
                node_dir.to_str().unwrap(),
                "--strip-components=1",
            ])
            .status()
            .map_err(|e| format!("解压失败: {}", e))?;

        if !status.success() {
            return Err("解压失败".into());
        }
    }

    // 验证安装
    let (node_path, npm_path) = find_nodejs_bin().ok_or("Node.js 安装后验证失败")?;
    let node_ver = Command::new(&node_path).arg("--version").output()
        .map_err(|e| format!("验证失败: {}", e))?;
    let npm_ver = Command::new(&npm_path).arg("--version").output()
        .map_err(|e| format!("验证失败: {}", e))?;

    if node_ver.status.success() && npm_ver.status.success() {
        let v = String::from_utf8_lossy(&node_ver.stdout).trim().to_string();
        emit_log(app, &format!("[✓] Node.js 安装完成 ({})", v));
    } else {
        return Err("Node.js 安装验证失败".into());
    }

    Ok(())
}

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

    let mut cmd = Command::new(&npm_path);
    cmd.arg("install").arg("-g").arg("@anthropic-ai/claude-code");

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
    emit_log(&app, "[ℹ] 提示: 如果命令行中无法使用 claude，请重启终端或手动将 Node.js 路径添加到 PATH。");

    Ok(())
}
