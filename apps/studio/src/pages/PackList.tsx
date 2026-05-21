import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { fetchPacks, checkHealth, type PackSummary } from '../lib/api'

export default function PackList() {
  const [packs, setPacks] = useState<PackSummary[]>([])
  const [loading, setLoading] = useState(true)
  const [runtimeStatus, setRuntimeStatus] = useState<string>('checking')

  useEffect(() => {
    checkHealth().then((h) => setRuntimeStatus(h.status))
    fetchPacks()
      .then(setPacks)
      .catch(() => {
        // Runtime not available yet, show empty state
      })
      .finally(() => setLoading(false))
  }, [])

  return (
    <div style={{ padding: 32, maxWidth: 900 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 32 }}>
        <div>
          <h1 style={{ fontSize: 24, fontWeight: 600, color: '#fff' }}>Scenario Packs</h1>
          <p style={{ fontSize: 13, color: '#888', marginTop: 4 }}>
            Runtime: <span style={{ color: runtimeStatus === 'ok' ? '#4ade80' : '#f87171' }}>{runtimeStatus}</span>
          </p>
        </div>
        <button
          style={{
            padding: '8px 16px',
            borderRadius: 6,
            border: 'none',
            background: '#3b82f6',
            color: '#fff',
            cursor: 'pointer',
            fontWeight: 500,
          }}
          onClick={() => {
            // TODO: Create new pack
          }}
        >
          New Pack
        </button>
      </div>

      {loading ? (
        <p style={{ color: '#888' }}>Loading...</p>
      ) : packs.length === 0 ? (
        <div style={{
          border: '1px dashed #333',
          borderRadius: 12,
          padding: 48,
          textAlign: 'center',
          color: '#888',
        }}>
          <p style={{ fontSize: 16, marginBottom: 8 }}>No scenario packs found</p>
          <p style={{ fontSize: 13 }}>
            Create a new pack or copy one to the <code style={{ background: '#1e1e1e', padding: '2px 6px', borderRadius: 4 }}>scenarios/</code> directory
          </p>
        </div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {packs.map((pack) => (
            <Link
              key={pack.id}
              to={`/pack/${encodeURIComponent(pack.path)}`}
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                padding: '16px 20px',
                borderRadius: 8,
                border: '1px solid #2a2a2a',
                textDecoration: 'none',
                color: '#e0e0e0',
                transition: 'border-color 0.15s',
              }}
              onMouseEnter={(e) => (e.currentTarget.style.borderColor = '#3b82f6')}
              onMouseLeave={(e) => (e.currentTarget.style.borderColor = '#2a2a2a')}
            >
              <div>
                <div style={{ fontWeight: 500, color: '#fff' }}>{pack.name}</div>
                <div style={{ fontSize: 13, color: '#888', marginTop: 2 }}>
                  {pack.id} &middot; v{pack.version}
                  {pack.description && <> &middot; {pack.description}</>}
                </div>
              </div>
              <span style={{
                fontSize: 12,
                padding: '2px 8px',
                borderRadius: 12,
                background: '#1e1e1e',
                color: '#888',
              }}>
                {pack.status || 'active'}
              </span>
            </Link>
          ))}
        </div>
      )}
    </div>
  )
}
