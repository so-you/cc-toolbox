export default function TopBar() {
  return (
    <header className="flex justify-between items-center px-8 w-full h-14 bg-surface-container-lowest/80 backdrop-blur-md border-b border-outline-variant flex-shrink-0 sticky top-0 z-10">
      <span className="text-h3 text-primary font-semibold">CC 工具箱</span>
      <div className="flex items-center gap-4 text-on-surface-variant">
        <span className="material-symbols-outlined cursor-pointer hover:opacity-70">desktop_windows</span>
        <span className="material-symbols-outlined cursor-pointer hover:opacity-70">desktop_mac</span>
      </div>
    </header>
  )
}
