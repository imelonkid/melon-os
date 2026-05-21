import { useState, useEffect, useCallback, useRef } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import Editor from '@monaco-editor/react'
import {
  runPack, getTask, getTraces, getApprovals, resolveApproval,
  type TraceEvent, type ApprovalItem
} from '../lib/api'

const RUNTIME_URL = ''

interface PackFileInfo {
  name: string
  path: string
}

const QUICK_TABS = [
  'manifest.yaml',
  'role.md',
  'tools/tools.yaml',
  'permissions/policy.yaml',
  'knowledge/sources.yaml',
  'ui/layout.yaml',
  'evals/cases.yaml',
] as const

function getLang(fileName: string): string {
  if (fileName.endsWith('.yaml') || fileName.endsWith('.yml')) return 'yaml'
  if (fileName.endsWith('.md')) return 'markdown'
  if (fileName.endsWith('.json')) return 'json'
  return 'plaintext'
}

const STATUS_COLORS: Record<string, string> = {
  created: '#888',
  running: '#3b82f6',
  completed: '#4ade80',
  failed: '#f87171',
  cancelled: '#f59e0b',
}

export default function PackEditor() {
  const { packPath } = useParams()
  const navigate = useNavigate()
  const [packId, setPackId] = useState<string>('')
  const [files, setFiles] = useState<Map<string, string>>(new Map())
  const [fileList, setFileList] = useState<PackFileInfo[]>([])
  const [activeFile, setActiveFile] = useState<string | null>(null)
  const [validationErrors, setValidationErrors] = useState<string[]>([])
  const [validating, setValidating] = useState(false)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)

  // Run/Debug state
  const [taskId, setTaskId] = useState<string | null>(null)
  const [taskStatus, setTaskStatus] = useState<string>('')
  const [traces, setTraces] = useState<TraceEvent[]>([])
  const [approvals, setApprovals] = useState<ApprovalItem[]>([])
  const [showDebug, setShowDebug] = useState(false)
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null)

  // Discover pack ID from URL or fetch from packs list
  useEffect(() => {
    if (!packPath) return
    fetch(`${RUNTIME_URL}/api/packs`)
      .then(r => r.json())
      .then((packs: any[]) => {
        const found = packs.find(p => p.path.includes(packPath || '') || p.id === packPath)
        if (found) {
          setPackId(found.id)
          loadPackFiles(found.id)
        } else if (packs.length > 0) {
          setPackId(packs[0].id)
          loadPackFiles(packs[0].id)
        }
      })
      .catch(() => {
        setPackId('demo.ops')
        loadPackFiles('demo.ops')
      })
  }, [packPath])

  // Poll for task updates when running
  useEffect(() => {
    if (pollRef.current) {
      clearInterval(pollRef.current)
      pollRef.current = null
    }
    if (taskId && ['created', 'running', 'awaiting_approval'].includes(taskStatus)) {
      pollRef.current = setInterval(() => {
        refreshTask()
      }, 800)
    }
    return () => {
      if (pollRef.current) {
        clearInterval(pollRef.current)
        pollRef.current = null
      }
    }
  }, [taskId, taskStatus])

  async function loadPackFiles(id: string) {
    setLoading(true)
    try {
      let discoveredFiles: PackFileInfo[] = []
      const listRes = await fetch(`${RUNTIME_URL}/api/packs/${id}/files`)
      if (listRes.ok) {
        const list: PackFileInfo[] = await listRes.json()
        discoveredFiles = list
        setFileList(list)
      }

      const loadedFiles = new Map<string, string>()
      for (const tab of QUICK_TABS) {
        try {
          const res = await fetch(`${RUNTIME_URL}/api/packs/${id}/files/${tab}`)
          if (res.ok) {
            const data = await res.json()
            loadedFiles.set(tab, data.content || '')
          }
        } catch {
          // Missing optional files
        }
      }
      setFiles(loadedFiles)
      setActiveFile(loadedFiles.has('manifest.yaml') ? 'manifest.yaml' : (discoveredFiles[0]?.path || null))
    } finally {
      setLoading(false)
    }
  }

  const handleEditorChange = useCallback((value: string | undefined) => {
    if (activeFile && value !== undefined) {
      setFiles((prev) => {
        const next = new Map(prev)
        next.set(activeFile, value)
        return next
      })
    }
  }, [activeFile])

  const handleValidate = useCallback(async () => {
    if (!packId) return
    setValidating(true)
    setValidationErrors([])
    try {
      const res = await fetch(`${RUNTIME_URL}/api/packs/${packId}/validate`, {
        method: 'POST',
      })
      if (res.ok) {
        const data = await res.json()
        setValidationErrors(data.errors || [])
      } else {
        setValidationErrors(['Failed to connect to Runtime for validation'])
      }
    } finally {
      setValidating(false)
    }
  }, [packId])

  const handleSave = useCallback(async () => {
    if (!packId) return
    setSaving(true)
    try {
      for (const [path, content] of files) {
        const res = await fetch(`${RUNTIME_URL}/api/packs/${packId}/files/${path}`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ content }),
        })
        if (!res.ok) {
          throw new Error(`Failed to save ${path}: ${res.status}`)
        }
      }
      alert('Pack saved')
    } catch (e: any) {
      alert(e?.message || 'Failed to save pack')
    } finally {
      setSaving(false)
    }
  }, [packId, files])

  const handleRun = useCallback(async () => {
    if (!packId) return
    try {
      const result = await runPack(packId, 'Execute workflow')
      setTaskId(result.task_id)
      setTaskStatus(result.status)
      setShowDebug(true)
      refreshTask()
    } catch (e: any) {
      alert(e?.message || 'Failed to run pack')
    }
  }, [packId])

  const refreshTask = useCallback(async () => {
    if (!taskId) return
    try {
      const [task, traceList, approvalList] = await Promise.all([
        getTask(taskId),
        getTraces(taskId),
        getApprovals(taskId),
      ])
      setTaskStatus(task.status)
      setTraces(traceList)
      setApprovals(approvalList.filter(a => a.status === 'pending'))
    } catch {
      // Task may not exist yet
    }
  }, [taskId])

  const handleApproval = useCallback(async (approvalId: string, approve: boolean) => {
    if (!taskId) return
    try {
      await resolveApproval(taskId, approvalId, approve)
      setApprovals(prev => prev.filter(a => a.id !== approvalId))
      refreshTask()
    } catch (e: any) {
      alert(e?.message || 'Failed to resolve approval')
    }
  }, [taskId, refreshTask])

  const currentLang = activeFile ? getLang(activeFile) : 'yaml'

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
      {/* Main area: sidebar + editor */}
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* File tree sidebar */}
        <aside style={{
          width: 220,
          borderRight: '1px solid #2a2a2a',
          background: '#141414',
          padding: '12px 0',
          overflowY: 'auto',
          display: 'flex',
          flexDirection: 'column',
          flexShrink: 0,
        }}>
          <div style={{ padding: '0 12px', marginBottom: 8 }}>
            <button
              onClick={() => navigate('/')}
              style={{
                background: 'none',
                border: 'none',
                color: '#888',
                cursor: 'pointer',
                fontSize: 13,
                padding: '4px 0',
              }}
            >
              &larr; Back
            </button>
          </div>

          {/* Quick tabs */}
          <div style={{ padding: '0 8px', fontSize: 11, color: '#666', marginBottom: 4 }}>Quick Edit</div>
          {QUICK_TABS.map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveFile(tab)}
              disabled={!files.has(tab)}
              style={{
                display: 'block',
                width: '100%',
                textAlign: 'left',
                padding: '6px 16px',
                border: 'none',
                background: activeFile === tab ? '#1e1e1e' : 'transparent',
                color: !files.has(tab) ? '#444' : activeFile === tab ? '#fff' : '#888',
                cursor: files.has(tab) ? 'pointer' : 'not-allowed',
                fontSize: 13,
                fontFamily: 'monospace',
                borderLeft: activeFile === tab ? '2px solid #3b82f6' : '2px solid transparent',
              }}
            >
              {tab}
            </button>
          ))}

          {/* All files */}
          {fileList.length > 0 && (
            <>
              <div style={{ padding: '12px 8px 4px', fontSize: 11, color: '#666' }}>All Files</div>
              {fileList.map((f) => (
                <button
                  key={f.path}
                  onClick={() => {
                    setActiveFile(f.path)
                    if (!files.has(f.path) && packId) {
                      fetch(`${RUNTIME_URL}/api/packs/${packId}/files/${f.path}`)
                        .then(r => r.ok ? r.json() : null)
                        .then(data => {
                          if (data) {
                            setFiles(prev => {
                              const next = new Map(prev)
                              next.set(f.path, data.content || '')
                              return next
                            })
                          }
                        })
                    }
                  }}
                  style={{
                    display: 'block',
                    width: '100%',
                    textAlign: 'left',
                    padding: '4px 16px',
                    border: 'none',
                    background: activeFile === f.path ? '#1e1e1e' : 'transparent',
                    color: activeFile === f.path ? '#fff' : '#666',
                    cursor: 'pointer',
                    fontSize: 12,
                    fontFamily: 'monospace',
                    borderLeft: activeFile === f.path ? '2px solid #3b82f6' : '2px solid transparent',
                  }}
                >
                  {f.path}
                </button>
              ))}
            </>
          )}

          {/* Actions */}
          <div style={{ padding: '16px 12px', borderTop: '1px solid #2a2a2a', marginTop: 'auto' }}>
            {/* Run button */}
            <button
              onClick={handleRun}
              disabled={['running', 'awaiting_approval'].includes(taskStatus) || !packId}
              style={{
                width: '100%',
                padding: '6px 12px',
                borderRadius: 6,
                border: 'none',
                background: '#22c55e',
                color: '#fff',
                cursor: ['running', 'awaiting_approval'].includes(taskStatus) ? 'wait' : 'pointer',
                fontSize: 13,
                fontWeight: 500,
                marginBottom: 8,
              }}
            >
              {taskStatus === 'awaiting_approval' ? 'Awaiting Approval...' : taskStatus === 'running' ? 'Running...' : 'Run'}
            </button>
            <button
              onClick={handleValidate}
              disabled={validating || !packId}
              style={{
                width: '100%',
                padding: '6px 12px',
                borderRadius: 6,
                border: '1px solid #333',
                background: '#1e1e1e',
                color: '#e0e0e0',
                cursor: validating ? 'wait' : 'pointer',
                fontSize: 13,
                marginBottom: 8,
              }}
            >
              {validating ? 'Validating...' : 'Validate'}
            </button>
            <button
              onClick={handleSave}
              disabled={saving || !packId}
              style={{
                width: '100%',
                padding: '6px 12px',
                borderRadius: 6,
                border: 'none',
                background: '#3b82f6',
                color: '#fff',
                cursor: saving ? 'wait' : 'pointer',
                fontSize: 13,
              }}
            >
              {saving ? 'Saving...' : 'Save'}
            </button>
            {validationErrors.length > 0 && (
              <div style={{ marginTop: 8, fontSize: 12, color: '#f87171' }}>
                {validationErrors.map((err, i) => (
                  <div key={i} style={{ marginBottom: 4 }}>{err}</div>
                ))}
              </div>
            )}
            {validationErrors.length === 0 && !validating && (
              <div style={{ marginTop: 8, fontSize: 12, color: '#4ade80' }}>
                Validation passed
              </div>
            )}
          </div>
        </aside>

        {/* Editor area */}
        <div style={{ flex: 1, overflow: 'hidden' }}>
          {loading ? (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: '#888' }}>
              Loading pack files...
            </div>
          ) : activeFile ? (
            <Editor
              height="100%"
              language={currentLang}
              value={files.get(activeFile) || ''}
              onChange={handleEditorChange}
              theme="vs-dark"
              options={{
                minimap: { enabled: false },
                fontSize: 14,
                padding: { top: 12, bottom: 12 },
                scrollBeyondLastLine: false,
                tabSize: 2,
              }}
            />
          ) : (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: '#888' }}>
              Select a file to edit
            </div>
          )}
        </div>
      </div>

      {/* Debug panel (bottom) */}
      {showDebug && (
        <div style={{
          borderTop: '1px solid #2a2a2a',
          background: '#141414',
          height: 280,
          display: 'flex',
          flexDirection: 'column',
        }}>
          {/* Panel header */}
          <div style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '8px 16px',
            borderBottom: '1px solid #2a2a2a',
          }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
              <span style={{ fontSize: 13, fontWeight: 600, color: '#fff' }}>Run / Debug</span>
              {taskId && (
                <span style={{
                  fontSize: 11,
                  padding: '2px 8px',
                  borderRadius: 10,
                  background: `${STATUS_COLORS[taskStatus] || '#888'}22`,
                  color: STATUS_COLORS[taskStatus] || '#888',
                  border: `1px solid ${STATUS_COLORS[taskStatus] || '#888'}44`,
                }}>
                  {taskStatus}
                </span>
              )}
            </div>
            <button
              onClick={() => setShowDebug(false)}
              style={{
                background: 'none',
                border: 'none',
                color: '#888',
                cursor: 'pointer',
                fontSize: 16,
                padding: '0 4px',
              }}
            >
              &times;
            </button>
          </div>

          {/* Panel body: traces + approvals side by side */}
          <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
            {/* Trace events */}
            <div style={{ flex: 1, overflowY: 'auto', padding: '8px 12px' }}>
              <div style={{ fontSize: 11, color: '#666', marginBottom: 6 }}>Trace Events</div>
              {traces.map((t, i) => {
                const color = t.event_type === 'system' ? '#888'
                  : t.event_type === 'tool' ? '#60a5fa'
                  : t.event_type === 'agent' ? '#a78bfa'
                  : t.event_type === 'approval' ? '#f59e0b'
                  : '#888'
                return (
                  <div key={t.id} style={{
                    display: 'flex',
                    gap: 8,
                    padding: '3px 0',
                    fontSize: 12,
                    fontFamily: 'monospace',
                    color: '#ccc',
                  }}>
                    <span style={{ color: '#555', minWidth: 16 }}>{i + 1}</span>
                    <span style={{ color, minWidth: 50, textTransform: 'uppercase', fontSize: 10 }}>{t.event_type}</span>
                    <span style={{ flex: 1, wordBreak: 'break-all' }}>{t.summary}</span>
                  </div>
                )
              })}
              {traces.length === 0 && (
                <div style={{ fontSize: 12, color: '#666', padding: 8 }}>Waiting for events...</div>
              )}
            </div>

            {/* Approvals */}
            {approvals.length > 0 && (
              <div style={{
                width: 280,
                borderLeft: '1px solid #2a2a2a',
                padding: '8px 12px',
                overflowY: 'auto',
              }}>
                <div style={{ fontSize: 11, color: '#666', marginBottom: 6 }}>Pending Approvals</div>
                {approvals.map(a => (
                  <div key={a.id} style={{
                    background: '#1e1e1e',
                    borderRadius: 6,
                    padding: 10,
                    marginBottom: 8,
                    border: '1px solid #333',
                  }}>
                    <div style={{ fontSize: 12, color: '#fff', marginBottom: 4 }}>{a.action}</div>
                    <div style={{ fontSize: 11, color: '#888', marginBottom: 8 }}>
                      Risk: <span style={{
                        color: a.risk_level === 'high' ? '#f87171' : a.risk_level === 'medium' ? '#f59e0b' : '#4ade80',
                      }}>{a.risk_level}</span>
                      {' '}&middot; Scope: {a.scope}
                    </div>
                    <div style={{ display: 'flex', gap: 6 }}>
                      <button
                        onClick={() => handleApproval(a.id, true)}
                        style={{
                          flex: 1,
                          padding: '4px 8px',
                          borderRadius: 4,
                          border: 'none',
                          background: '#22c55e',
                          color: '#fff',
                          cursor: 'pointer',
                          fontSize: 12,
                        }}
                      >
                        Approve
                      </button>
                      <button
                        onClick={() => handleApproval(a.id, false)}
                        style={{
                          flex: 1,
                          padding: '4px 8px',
                          borderRadius: 4,
                          border: 'none',
                          background: '#ef4444',
                          color: '#fff',
                          cursor: 'pointer',
                          fontSize: 12,
                        }}
                      >
                        Reject
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
