import { Outlet } from 'react-router-dom'
import SideNav from './SideNav'
import TopBar from './TopBar'

export default function Layout() {
  return (
    <div className="flex h-screen overflow-hidden bg-background text-on-background">
      <SideNav />
      <div className="flex-1 flex flex-col min-w-0 relative z-10">
        <TopBar />
        <main className="flex-1 overflow-y-auto p-margin-page">
          <Outlet />
        </main>
      </div>
    </div>
  )
}
