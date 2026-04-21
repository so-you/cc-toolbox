import { useState, useEffect, useRef } from 'react'
import { invokeCommand, listenEvent } from '../hooks/useTauriCommand'
import { SystemStatus } from '../types'
import StatusBadge from '../components/StatusBadge'

export default function InstallPage() {
  const [status, setStatus] = useState<SystemStatus | null>(null)
  const [installing, setInstalling] = useState(false)
  const [logs, setLogs] = useState<string[]>([])
  const [mirror, setMirror] = useState(true)
  const logEndRef = useRef<HTMLDivElement>(null)

  const runCheck = async () => {
    try {
      const result = await invokeCommand<SystemStatus>('system_check')
      setStatus(result)
    } catch (e) {
      console.error(e)
    }
  }

  const runInstall = async () => {
    setInstalling(true)
    setLogs([])
    const unlisten = await listenEvent<{ line: string }>('install-log', (payload) => {
      setLogs((prev) => [...prev, payload.line])
    })
    try {
      await invokeCommand('install_claude_code', {
        mirror: mirror ? 'taobao' : 'default',
      })
      setLogs((prev) => [...prev, '[✓] 安装完成'])
      runCheck()
    } catch (e: any) {
      setLogs((prev) => [...prev, `[✗] 安装失败: ${e}`])
    } finally {
      setInstalling(false)
      unlisten()
    }
  }

  useEffect(() => {
    runCheck()
  }, [])

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [logs])

  const allReady = status?.node?.installed && status?.git?.installed && status?.npm?.installed

  return (
    <div className="max-w-6xl mx-auto">
      <div className="mb-xl">
        <h1 className="text-h1 text-on-surface mb-xs">环境配置与安装</h1>
        <p className="text-body-lg text-on-surface-variant">一键检测系统环境并安装 Claude Code，让终端 AI 助手即刻就绪。</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-gutter">
        <div className="lg:col-span-2 bg-surface-container-lowest rounded-xl border border-surface-variant shadow-sm flex flex-col overflow-hidden relative">
          <div className="p-xl relative z-10 flex flex-col items-center justify-center border-b border-surface-variant bg-surface-container-lowest text-center">
            <div className="w-16 h-16 rounded-full bg-surface-container flex items-center justify-center mb-md border border-surface-variant shadow-sm">
              <span className="material-symbols-outlined text-3xl text-on-surface">code_blocks</span>
            </div>
            <h3 className="text-h2 text-on-surface mb-sm">安装 Claude Code</h3>
            <p className="text-body-md text-on-surface-variant mb-lg max-w-md">此操作将通过 npm 全局安装 @anthropic-ai/claude-code。请确保下方环境检测均已通过。</p>
            <button
              onClick={runInstall}
              disabled={!allReady || installing}
              className="bg-primary text-on-primary text-button h-12 px-8 rounded-lg flex items-center gap-2 hover:bg-surface-tint transition-colors shadow-md active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <span className="material-symbols-outlined text-[20px]">{installing ? 'sync' : 'download'}</span>
              {installing ? '安装中...' : '一键开始安装'}
            </button>
          </div>

          <div className="p-lg relative z-10 bg-surface-container-lowest/80 flex-1">
            <h4 className="text-h3 text-on-surface mb-md flex items-center gap-2">
              <span className="material-symbols-outlined text-[20px] text-on-surface-variant">checklist</span>
              环境检测进度
            </h4>
            <div className="flex flex-col gap-0 border border-surface-variant rounded-lg overflow-hidden bg-surface-container-lowest">
              <StatusBadge status={status?.node ?? null} label="Node.js 环境" />
              <StatusBadge status={status?.git ?? null} label="Git 版本控制" />
              <StatusBadge status={status?.npm ?? null} label="NPM 包管理器" />
              <div className="flex items-center justify-between p-4 opacity-60">
                <div className="flex items-center gap-4">
                  <span className="material-symbols-outlined text-outline">{status?.claude_code?.installed ? 'check_circle' : 'pending'}</span>
                  <div>
                    <div className="text-h3 text-on-surface">全局依赖安装</div>
                    <div className="text-label-sm text-on-surface-variant mt-xs">npm install -g @anthropic-ai/claude-code</div>
                  </div>
                </div>
                <div className="bg-surface-container py-1 px-3 rounded-full border border-surface-variant border-dashed">
                  <span className="text-label-sm text-on-surface-variant">{status?.claude_code?.installed ? '已安装' : '等待执行'}</span>
                </div>
              </div>
            </div>

            {logs.length > 0 && (
              <div className="mt-md border border-surface-variant rounded-lg bg-black text-white p-4 font-mono text-xs h-48 overflow-y-auto">
                {logs.map((line, i) => (
                  <div key={i} className="whitespace-pre-wrap">{line}</div>
                ))}
                <div ref={logEndRef} />
              </div>
            )}
          </div>
        </div>

        <div className="lg:col-span-1 flex flex-col gap-gutter">
          <div className="bg-surface-container-lowest rounded-xl border border-surface-variant p-lg shadow-sm">
            <h4 className="text-h3 text-on-surface mb-md">安装选项</h4>
            <label className="flex items-start gap-3 cursor-pointer group mb-4">
              <div className="relative flex items-center justify-center mt-1">
                <input type="checkbox" checked={mirror} onChange={(e) => setMirror(e.target.checked)} className="peer sr-only" />
                <div className="w-5 h-5 rounded border border-outline peer-checked:bg-primary peer-checked:border-primary transition-colors"></div>
                <span className="material-symbols-outlined absolute text-on-primary text-[16px] opacity-0 peer-checked:opacity-100 transition-opacity pointer-events-none" style={{ fontVariationSettings: "'wght' 600" }}>check</span>
              </div>
              <div>
                <div className="text-body-md text-on-surface font-medium group-hover:text-primary transition-colors">使用 NPM 镜像加速</div>
                <div className="text-label-sm text-on-surface-variant mt-1">推荐国内用户开启，提升下载速度。默认使用淘宝镜像。</div>
              </div>
            </label>
          </div>
        </div>
      </div>
    </div>
  )
}
