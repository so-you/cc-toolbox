use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use tauri::{AppHandle, Emitter};

#[derive(Clone, serde::Serialize)]
struct InstallLogEvent {
    line: String,
}

pub fn install_claude_code(app: AppHandle, mirror: String) -> Result<(), String> {
    let registry_arg = match mirror.as_str() {
        "taobao" => "--registry=https://registry.npmmirror.com",
        _ => "",
    };

    let mut cmd = Command::new("npm");
    cmd.arg("install")
        .arg("-g")
        .arg("@anthropic-ai/claude-code");

    if !registry_arg.is_empty() {
        cmd.arg(registry_arg);
    }

    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn npm: {}", e))?;

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

    let status = child.wait().map_err(|e| format!("Failed to wait on child: {}", e))?;
    if !status.success() {
        return Err(format!("npm install exited with code: {:?}", status.code()));
    }

    Ok(())
}
