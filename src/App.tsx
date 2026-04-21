import { HashRouter, Routes, Route } from 'react-router-dom'
import Layout from './components/Layout'
import InstallPage from './pages/InstallPage'
import ApiConfigPage from './pages/ApiConfigPage'
import AliasPage from './pages/AliasPage'

export default function App() {
  return (
    <HashRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<InstallPage />} />
          <Route path="api-config" element={<ApiConfigPage />} />
          <Route path="aliases" element={<AliasPage />} />
        </Route>
      </Routes>
    </HashRouter>
  )
}
