use crate::models::Provider;
use std::fs;
use std::path::PathBuf;

const ALIAS_START: &str = "# === cc-toolbox aliases START ===";
const ALIAS_END: &str = "# === cc-toolbox aliases END ===";

fn shell_profile_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("could not determine home directory")?;

    let shell = std::env::var("SHELL").unwrap_or_default();
    if shell.contains("zsh") || !home.join(".bashrc").exists() {
        Ok(home.join(".zshrc"))
    } else {
        Ok(home.join(".bashrc"))
    }
}

fn generate_alias_block(providers: &[Provider]) -> String {
    let mut lines = vec![ALIAS_START.to_string()];
    lines.push("alias cc='claude'".to_string());

    for p in providers {
        if !p.enabled || p.api_key.is_empty() {
            continue;
        }
        let alias = match p.id.as_str() {
            "anthropic" => "cc",
            "openai" => "gpt",
            "deepseek" => "dpk",
            "kimi" => "kimi",
            "glm" => "glm",
            _ => &p.id,
        };
        lines.push(format!(
            "alias {}='ANTHROPIC_API_KEY={} ANTHROPIC_BASE_URL={} ANTHROPIC_MODEL={} claude'",
            alias, p.api_key, p.base_url, p.model
        ));
    }

    lines.push(ALIAS_END.to_string());
    lines.join("\n")
}

pub fn write_aliases(providers: &[Provider]) -> Result<String, String> {
    let profile = shell_profile_path()?;
    let block = generate_alias_block(providers);

    let content = if profile.exists() {
        fs::read_to_string(&profile).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    if profile.exists() {
        let backup = profile.with_extension("cc-toolbox-backup");
        fs::copy(&profile, &backup).map_err(|e| e.to_string())?;
    }

    let new_content = if content.contains(ALIAS_START) && content.contains(ALIAS_END) {
        let start = content.find(ALIAS_START).unwrap();
        let end = content.find(ALIAS_END).unwrap() + ALIAS_END.len();
        format!("{}{}\n{}", &content[..start], block, &content[end..])
    } else {
        format!("{}\n{}\n", content.trim_end(), block)
    };

    fs::write(&profile, new_content).map_err(|e| e.to_string())?;

    Ok(profile.to_string_lossy().to_string())
}
