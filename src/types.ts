export interface VersionStatus {
  installed: boolean
  version?: string
  required: string
  message: string
}

export interface SystemStatus {
  node: VersionStatus | null
  git: VersionStatus | null
  npm: VersionStatus | null
  claude_code: VersionStatus | null
}

export interface Provider {
  id: string
  name: string
  api_key: string
  base_url: string
  model: string
  enabled: boolean
  updated_at?: string
}

export interface Settings {
  mirror: string
  auto_alias: boolean
}

export interface AppConfig {
  providers: Provider[]
  settings: Settings
}

export const DEFAULT_PROVIDERS: Provider[] = [
  { id: 'anthropic', name: 'Anthropic', api_key: '', base_url: 'https://api.anthropic.com', model: 'claude-sonnet-4-6', enabled: false },
  { id: 'openai', name: 'OpenAI', api_key: '', base_url: 'https://api.openai.com/v1', model: 'gpt-4o', enabled: false },
  { id: 'deepseek', name: 'DeepSeek', api_key: '', base_url: 'https://api.deepseek.com', model: 'deepseek-chat', enabled: false },
  { id: 'kimi', name: 'Kimi', api_key: '', base_url: 'https://api.moonshot.cn/v1', model: 'moonshot-v1-8k', enabled: false },
  { id: 'glm', name: '智谱 GLM', api_key: '', base_url: 'https://open.bigmodel.cn/api/paas/v4/', model: 'glm-4-plus', enabled: false },
]
