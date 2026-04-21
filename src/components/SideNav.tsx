import { Link, useLocation } from 'react-router-dom'

const navItems = [
  { to: '/', icon: 'home', label: '首页' },
  { to: '/api-config', icon: 'vpn_key', label: 'API 配置' },
  { to: '/aliases', icon: 'terminal', label: '终端别名' },
]

export default function SideNav() {
  const location = useLocation()

  return (
    <nav className="h-screen w-64 border-r border-outline-variant bg-surface-container-lowest flex-shrink-0 z-20">
      <div className="flex flex-col h-full p-4">
        <div className="flex items-center gap-3 px-4 py-4 mb-4">
          <div className="w-8 h-8 rounded-lg bg-primary text-on-primary flex items-center justify-center flex-shrink-0 font-bold text-sm">
            CC
          </div>
          <div>
            <h2 className="text-base font-semibold text-primary tracking-tight">CC 工具箱</h2>
            <p className="text-label-sm text-on-surface-variant">Claude Code 助手</p>
          </div>
        </div>
        <ul className="flex flex-col gap-1 flex-1">
          {navItems.map((item) => {
            const active = location.pathname === item.to
            return (
              <li key={item.to}>
                <Link
                  to={item.to}
                  className={
                    'flex items-center gap-3 px-4 py-3 rounded-lg font-button text-button transition-all duration-200 active:scale-95 ' +
                    (active
                      ? 'bg-surface-container-highest text-primary font-bold'
                      : 'text-on-surface-variant hover:bg-surface-container-low')
                  }
                >
                  <span className="material-symbols-outlined">{item.icon}</span>
                  <span>{item.label}</span>
                </Link>
              </li>
            )
          })}
        </ul>
      </div>
    </nav>
  )
}
