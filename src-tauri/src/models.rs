use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub node: Option<VersionStatus>,
    pub git: Option<VersionStatus>,
    pub npm: Option<VersionStatus>,
    pub claude_code: Option<VersionStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub required: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub enabled: bool,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub providers: Vec<Provider>,
    pub settings: Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub mirror: String,
    pub auto_alias: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            providers: vec![
                Provider {
                    id: "anthropic".into(),
                    name: "Anthropic".into(),
                    api_key: String::new(),
                    base_url: "https://api.anthropic.com".into(),
                    model: "claude-sonnet-4-6".into(),
                    enabled: false,
                    updated_at: None,
                },
                Provider {
                    id: "openai".into(),
                    name: "OpenAI".into(),
                    api_key: String::new(),
                    base_url: "https://api.openai.com/v1".into(),
                    model: "gpt-4o".into(),
                    enabled: false,
                    updated_at: None,
                },
                Provider {
                    id: "deepseek".into(),
                    name: "DeepSeek".into(),
                    api_key: String::new(),
                    base_url: "https://api.deepseek.com".into(),
                    model: "deepseek-chat".into(),
                    enabled: false,
                    updated_at: None,
                },
                Provider {
                    id: "kimi".into(),
                    name: "Kimi".into(),
                    api_key: String::new(),
                    base_url: "https://api.moonshot.cn/v1".into(),
                    model: "moonshot-v1-8k".into(),
                    enabled: false,
                    updated_at: None,
                },
                Provider {
                    id: "glm".into(),
                    name: "智谱 GLM".into(),
                    api_key: String::new(),
                    base_url: "https://open.bigmodel.cn/api/paas/v4/".into(),
                    model: "glm-4-plus".into(),
                    enabled: false,
                    updated_at: None,
                },
            ],
            settings: Settings {
                mirror: "taobao".into(),
                auto_alias: true,
            },
        }
    }
}
