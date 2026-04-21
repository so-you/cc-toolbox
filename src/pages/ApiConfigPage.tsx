import { useState, useEffect } from 'react'
import { invokeCommand } from '../hooks/useTauriCommand'
import { Provider } from '../types'

const ICONS: Record<string, string> = {
  anthropic: 'neurology',
  openai: 'all_inclusive',
  deepseek: 'radar',
  kimi: 'chat',
  glm: 'smart_toy',
}

function ProviderCard({ provider, onSave }: { provider: Provider; onSave: (p: Provider) => void }) {
  const [key, setKey] = useState(provider.api_key)
  const [baseUrl, setBaseUrl] = useState(provider.base_url)
  const [model, setModel] = useState(provider.model)
  const [showKey, setShowKey] = useState(false)
  const [saving, setSaving] = useState(false)

  const handleSave = async () => {
    setSaving(true)
    const updated: Provider = {
      ...provider,
      api_key: key,
      base_url: baseUrl,
      model: model,
      updated_at: new Date().toISOString(),
    }
    try {
      await invokeCommand('save_provider', { provider: updated })
      onSave(updated)
    } catch (e) {
      console.error(e)
    }
    setSaving(false)
  }

  const isDirty = key !== provider.api_key || baseUrl !== provider.base_url || model !== provider.model
  const hasKey = !!provider.api_key

  return (
    <div className={`bg-surface-container-lowest rounded-xl border border-outline-variant p-md flex flex-col md:flex-row gap-lg md:items-start transition-all hover:border-outline ${!hasKey ? 'opacity-75' : ''}`}>
      <div className="md:w-64 flex-shrink-0">
        <div className="flex items-center gap-3 mb-2">
          <div className="w-10 h-10 rounded-lg bg-surface-container flex items-center justify-center border border-surface-variant">
            <span className="material-symbols-outlined text-primary">{ICONS[provider.id] || 'api'}</span>
          </div>
          <h3 className="text-h3 text-primary">{provider.name}</h3>
        </div>
        <p className="text-body-md text-on-surface-variant">{provider.id === 'anthropic' ? 'Claude 3 系列模型的主要服务商。' : '支持多模型调用与自定义配置。'}</p>
      </div>
      <div className="flex-1 flex flex-col gap-sm">
        <label className="text-label-sm text-on-surface">API Key</label>
        <div className="flex gap-sm">
          <div className="relative flex-1">
            <span className="absolute inset-y-0 left-0 flex items-center pl-3 text-on-surface-variant">
              <span className="material-symbols-outlined" style={{ fontSize: 18 }}>key</span>
            </span>
            <input
              type={showKey ? 'text' : 'password'}
              value={key}
              onChange={(e) => setKey(e.target.value)}
              placeholder={provider.id === 'anthropic' ? 'sk-ant-...' : 'sk-...'}
              className="w-full bg-surface-container-lowest border border-outline-variant text-primary text-body-md pl-10 pr-10 py-2.5 rounded-lg focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary placeholder-on-surface-variant transition-colors"
            />
            <button
              onClick={() => setShowKey(!showKey)}
              className="absolute inset-y-0 right-0 flex items-center pr-3 text-on-surface-variant hover:text-primary"
            >
              <span className="material-symbols-outlined text-[18px]">{showKey ? 'visibility_off' : 'visibility'}</span>
            </button>
          </div>
          <button
            onClick={handleSave}
            disabled={!isDirty || saving}
            className={`px-md py-2.5 rounded-lg font-button text-button whitespace-nowrap flex-shrink-0 transition-opacity ${
              hasKey && !isDirty
                ? 'bg-surface-container-highest text-primary border border-outline-variant hover:bg-surface-dim'
                : 'bg-primary text-on-primary hover:opacity-90'
            } disabled:opacity-50`}
          >
            {saving ? '保存中...' : '保存'}
          </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-sm mt-2">
          <div>
            <label className="text-label-sm text-on-surface block mb-1">Base URL</label>
            <input
              type="text"
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              className="w-full bg-surface-container-lowest border border-outline-variant text-primary text-body-md px-4 py-2.5 rounded-lg focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-colors"
            />
          </div>
          <div>
            <label className="text-label-sm text-on-surface block mb-1">模型 ID</label>
            <input
              type="text"
              value={model}
              onChange={(e) => setModel(e.target.value)}
              className="w-full bg-surface-container-lowest border border-outline-variant text-primary text-body-md px-4 py-2.5 rounded-lg focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-colors"
            />
          </div>
        </div>

        <p className="text-label-sm text-on-surface-variant mt-1">
          {provider.updated_at ? `最后更新: ${new Date(provider.updated_at).toLocaleString()}` : '未配置'}
        </p>
      </div>
    </div>
  )
}

export default function ApiConfigPage() {
  const [providers, setProviders] = useState<Provider[]>([])
  const [loading, setLoading] = useState(true)

  const load = async () => {
    setLoading(true)
    try {
      const result = await invokeCommand<Provider[]>('load_providers')
      setProviders(result)
    } catch (e) {
      console.error(e)
    }
    setLoading(false)
  }

  useEffect(() => {
    load()
  }, [])

  return (
    <div className="max-w-4xl mx-auto">
      <div className="mb-xl">
        <h1 className="text-h1 text-primary mb-sm">API 配置</h1>
        <p className="text-body-lg text-on-surface-variant">管理模型服务商的 API 密钥以启用各项功能。密钥将安全地存储在本地环境中。</p>
      </div>

      {loading ? (
        <div className="text-body-md text-on-surface-variant">加载中...</div>
      ) : (
        <div className="space-y-md">
          {providers.map((p) => (
            <ProviderCard key={p.id} provider={p} onSave={(updated) => {
              setProviders((prev) => prev.map((x) => (x.id === updated.id ? updated : x)))
            }} />
          ))}
        </div>
      )}
    </div>
  )
}
