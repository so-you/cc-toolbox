import { useState, useEffect } from 'react'
import { invokeCommand } from '../hooks/useTauriCommand'
import { Provider } from '../types'

const ALIAS_MAP: Record<string, string> = {
  anthropic: 'cc',
  openai: 'gpt',
  deepseek: 'dpk',
  kimi: 'kimi',
  glm: 'glm',
}

export default function AliasPage() {
  const [providers, setProviders] = useState<Provider[]>([])
  const [writing, setWriting] = useState(false)
  const [writeResult, setWriteResult] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  const load = async () => {
    try {
      const result = await invokeCommand<Provider[]>('load_providers')
      setProviders(result)
    } catch (e) {
      console.error(e)
    }
  }

  useEffect(() => {
    load()
  }, [])

  const toggleEnabled = async (id: string) => {
    const updated = providers.map((p) =>
      p.id === id ? { ...p, enabled: !p.enabled } : p
    )
    setProviders(updated)
    const target = updated.find((p) => p.id === id)
    if (target) {
      try {
        await invokeCommand('save_provider', { provider: target })
      } catch (e) {
        console.error(e)
      }
    }
  }

  const handleWrite = async () => {
    setWriting(true)
    setWriteResult(null)
    setError(null)
    try {
      const path = await invokeCommand<string>('write_aliases', { providers })
      setWriteResult(path)
    } catch (e: any) {
      setError(String(e))
    }
    setWriting(false)
  }

  return (
    <div className="max-w-4xl mx-auto">
      <div className="mb-xl">
        <h1 className="text-h1 text-primary mb-sm">终端别名</h1>
        <p className="text-body-lg text-on-surface-variant">管理已配置的模型别名，一键写入 shell 配置文件。</p>
      </div>

      <div className="space-y-md">
        {providers.map((p) => {
          const alias = ALIAS_MAP[p.id] || p.id
          const ready = !!p.api_key
          return (
            <div key={p.id} className={`bg-surface-container-lowest rounded-xl border border-outline-variant p-md flex items-center justify-between ${!ready ? 'opacity-50' : ''}`}>
              <div className="flex items-center gap-4">
                <div className="w-10 h-10 rounded-lg bg-surface-container flex items-center justify-center border border-surface-variant">
                  <span className="material-symbols-outlined text-primary">terminal</span>
                </div>
                <div>
                  <div className="text-h3 text-primary">
                    {p.name} <code className="text-body-md bg-surface-container px-2 py-0.5 rounded text-on-surface-variant ml-2">{alias}</code>
                  </div>
                  <div className="text-label-sm text-on-surface-variant mt-1">
                    {ready ? (p.enabled ? '已启用' : '已禁用') : '请先配置 API Key'}
                  </div>
                </div>
              </div>
              <button
                onClick={() => toggleEnabled(p.id)}
                disabled={!ready}
                className={`relative w-11 h-6 rounded-full transition-colors ${p.enabled ? 'bg-primary' : 'bg-surface-dim'} disabled:cursor-not-allowed`}
              >
                <span className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform ${p.enabled ? 'translate-x-5' : ''}`} />
              </button>
            </div>
          )
        })}
      </div>

      <div className="mt-xl flex flex-col items-start gap-sm">
        <button
          onClick={handleWrite}
          disabled={writing}
          className="bg-primary text-on-primary text-button h-12 px-8 rounded-lg flex items-center gap-2 hover:bg-surface-tint transition-colors shadow-md active:scale-[0.98] disabled:opacity-50"
        >
          <span className="material-symbols-outlined">save</span>
          {writing ? '写入中...' : '写入配置文件'}
        </button>

        {writeResult && (
          <div className="text-body-md text-primary">
            已写入: <code className="bg-surface-container px-2 py-1 rounded">{writeResult}</code>
          </div>
        )}
        {error && (
          <div className="text-body-md text-error">
            失败: {error}
          </div>
        )}
        {writeResult && (
          <p className="text-label-sm text-on-surface-variant">
            请运行 <code className="bg-surface-container px-1 rounded">source {writeResult}</code> 或重启终端使别名生效。
          </p>
        )}
      </div>
    </div>
  )
}
