import { VersionStatus } from '../types'

interface Props {
  status: VersionStatus | null
  label: string
}

export default function StatusBadge({ status, label }: Props) {
  if (!status) {
    return (
      <div className="flex items-center gap-4 p-4 border-b border-surface-variant">
        <span className="material-symbols-outlined text-outline">pending</span>
        <div>
          <div className="text-h3 text-on-surface">{label}</div>
          <div className="text-label-sm text-on-surface-variant mt-xs">检测中...</div>
        </div>
      </div>
    )
  }

  const icon = status.installed ? 'check_circle' : 'cancel'
  const color = status.installed ? 'text-primary' : 'text-error'
  const dotColor = status.installed ? 'bg-primary' : 'bg-error'

  return (
    <div className="flex items-center justify-between p-4 border-b border-surface-variant hover:bg-surface-container-low transition-colors">
      <div className="flex items-center gap-4">
        <span className={`material-symbols-outlined ${color}`} style={{ fontVariationSettings: "'FILL' 1" }}>{icon}</span>
        <div>
          <div className="text-h3 text-on-surface">{label}</div>
          <div className="text-label-sm text-on-surface-variant mt-xs">{status.message}</div>
        </div>
      </div>
      <div className="bg-surface-container py-1 px-3 rounded-full border border-surface-variant flex items-center gap-2">
        <span className={`w-2 h-2 rounded-full ${dotColor}`}></span>
        <span className="text-label-sm text-on-surface">{status.installed ? status.version || '已就绪' : '未安装'}</span>
      </div>
    </div>
  )
}
