import { BrowserRouter, Routes, Route, Link, useLocation } from 'react-router-dom'
import PackList from './pages/PackList'
import PackEditor from './pages/PackEditor'

function Layout() {
  const location = useLocation()
  const isEditor = location.pathname.startsWith('/pack/')

  return (
    <div style={{ display: 'flex', height: '100vh' }}>
      {!isEditor && (
        <aside style={{
          width: 220,
          borderRight: '1px solid #2a2a2a',
          padding: '16px',
          background: '#141414',
        }}>
          <h2 style={{ fontSize: 18, fontWeight: 600, marginBottom: 24, color: '#fff' }}>
            melon Studio
          </h2>
          <nav>
            <Link to="/" style={{
              display: 'block',
              padding: '8px 12px',
              borderRadius: 6,
              color: '#e0e0e0',
              textDecoration: 'none',
              background: '#1e1e1e',
              marginBottom: 4,
            }}>
              Scenario Packs
            </Link>
          </nav>
        </aside>
      )}
      <main style={{ flex: 1, overflow: 'auto' }}>
        <Routes>
          <Route path="/" element={<PackList />} />
          <Route path="/pack/:packPath/*" element={<PackEditor />} />
        </Routes>
      </main>
    </div>
  )
}

export default function App() {
  return (
    <BrowserRouter>
      <Layout />
    </BrowserRouter>
  )
}
