import { useState, useEffect, useCallback, useRef } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import Editor from '@monaco-editor/react'
import {
  runPack, getTask, getTraces, getApprovals, resolveApproval,
  getAuditLogs, runEval,
  type TraceEvent, type ApprovalItem, type AuditLogEntry, type EvalSummary
} from '../lib/api'
import Panels from '../components/Panels'
import { parseLayout, routePanels, type UiLayoutConfig, type RegionedPanels } from '../lib/panels'

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

const TRACE_TYPES = ['all', 'system', 'tool', 'agent', 'ui', 'knowledge', 'approval'] as const

const TRACE_TYPE_COLORS: Record<string, string> = {
  system: '#888',
  tool: '#60a5fa',
  agent: '#a78bfa',
  ui: '#34d399',
  knowledge: '#f59e0b',
  approval: '#f472b6',
}

function formatRefValue(ref: string): string {
  try {
    const parsed = JSON.parse(ref)
    return JSON.stringify(parsed, null, 2).slice(0, 300)
  } catch {
    return ref.slice(0, 300)
  }
}

function formatTraceTime(timestamp: string, firstTimestamp: string | null): string {
  if (!firstTimestamp) return ''
  const first = new Date(firstTimestamp).getTime()
  const current = new Date(timestamp).getTime()
  const diff = (current - first) / 1000
  if (diff < 60) return `+${diff.toFixed(1)}s`
  const mins = Math.floor(diff / 60)
  const secs = Math.floor(diff % 60)
  return `+${mins}m${secs}s`
}

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
  const [auditLogs, setAuditLogs] = useState<AuditLogEntry[]>([])
  const [evalResult, setEvalResult] = useState<EvalSummary | null>(null)
  const [evalRunning, setEvalRunning] = useState(false)
  const [debugTab, setDebugTab] = useState<'trace' | 'audit' | 'eval' | 'panels'>('trace')
  const [showDebug, setShowDebug] = useState(false)
  const [traceFilter, setTraceFilter] = useState<string>('all')
  const [expandedTrace, setExpandedTrace] = useState<string | null>(null)
  const [layoutConfig, setLayoutConfig] = useState<UiLayoutConfig | null>(null)
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null)

  // Parse ui/layout.yaml when files are loaded
  useEffect(() => {
    const layoutContent = files.get('ui/layout.yaml')
    setLayoutConfig(layoutContent ? parseLayout(layoutContent) : null)
  }, [files])

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
      setDebugTab('trace')
      setTraceFilter('all')
      setExpandedTrace(null)
      setAuditLogs([])
      setEvalResult(null)
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
      setTaskStatus(prev => {
        if (prev !== task.status) {
          setEvalResult(null)
        }
        return task.status
      })
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
      setEvalResult(null)
      refreshTask()
    } catch (e: any) {
      alert(e?.message || 'Failed to resolve approval')
    }
  }, [taskId, refreshTask])

  const handlePanelAction = useCallback((action: string, params?: Record<string, any>) => {
    const approvalId = typeof params?.approval_id === 'string' ? params.approval_id : ''
    if (!approvalId) return
    if (action === 'approval.approve') {
      handleApproval(approvalId, true)
    }
    if (action === 'approval.reject') {
      handleApproval(approvalId, false)
    }
  }, [handleApproval])

  const handleRunEval = useCallback(async () => {
    if (!taskId) return
    if (taskStatus !== 'completed') {
      setEvalResult(null)
      setDebugTab('eval')
      return
    }
    setEvalRunning(true)
    try {
      const result = await runEval(taskId)
      setEvalResult(result)
      setDebugTab('eval')
    } catch (e: any) {
      alert(e?.message || 'Failed to run eval')
    } finally {
      setEvalRunning(false)
    }
  }, [taskId, taskStatus])

  const handleLoadAudit = useCallback(async () => {
    if (!taskId) return
    try {
      const logs = await getAuditLogs(taskId)
      setAuditLogs(logs)
      setDebugTab('audit')
    } catch {
      // Audit endpoint may not be available
    }
  }, [taskId])

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
          height: 360,
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

          {/* Panel body: tabs for trace / audit / eval + approvals */}
          <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
            {/* Left side: tabbed content */}
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
              {/* Tabs */}
              <div style={{
                display: 'flex',
                gap: 0,
                borderBottom: '1px solid #2a2a2a',
                padding: '0 12px',
              }}>
                {(['trace', 'audit', 'eval', 'panels'] as const).map(tab => (
                  <button
                    key={tab}
                    onClick={() => {
                      if (tab === 'audit') handleLoadAudit()
                      if (tab === 'eval' && !evalResult) handleRunEval()
                      setDebugTab(tab)
                    }}
                    style={{
                      background: 'none',
                      border: 'none',
                      borderBottom: debugTab === tab ? '2px solid #3b82f6' : '2px solid transparent',
                      color: debugTab === tab ? '#fff' : '#888',
                      cursor: 'pointer',
                      fontSize: 12,
                      padding: '6px 12px',
                      textTransform: 'capitalize',
                    }}
                  >
                    {tab === 'eval' && evalRunning ? 'Running...' : tab}
                    {tab === 'eval' && evalResult && ` (${evalResult.passed}/${evalResult.total})`}
                    {tab === 'audit' && auditLogs.length > 0 && ` (${auditLogs.length})`}
                  </button>
                ))}
              </div>

              {/* Trace tab */}
              {debugTab === 'trace' && (
                <div style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column' }}>
                  {/* Filter bar */}
                  <div style={{ padding: '8px 12px 4px', display: 'flex', flexWrap: 'wrap', gap: 4 }}>
                    {TRACE_TYPES.map(type => (
                      <button
                        key={type}
                        onClick={() => setTraceFilter(type)}
                        style={{
                          padding: '3px 10px',
                          borderRadius: 12,
                          border: '1px solid',
                          borderColor: traceFilter === type
                            ? (type === 'all' ? '#3b82f6' : TRACE_TYPE_COLORS[type] || '#888')
                            : '#333',
                          background: traceFilter === type
                            ? ((type === 'all' ? '#3b82f6' : TRACE_TYPE_COLORS[type] || '#888') + '22')
                            : 'transparent',
                          color: traceFilter === type ? '#fff' : '#666',
                          cursor: 'pointer',
                          fontSize: 11,
                          textTransform: 'capitalize',
                        }}
                      >
                        {type}
                      </button>
                    ))}
                  </div>
                  {/* Trace list */}
                  <div style={{ flex: 1, overflowY: 'auto', padding: '4px 12px 8px' }}>
                    {(() => {
                      const filtered = traceFilter === 'all'
                        ? traces
                        : traces.filter(t => t.event_type === traceFilter)
                      const firstTs = filtered.length > 0 ? filtered[0].timestamp : null

                      return filtered.map((t) => {
                        const color = TRACE_TYPE_COLORS[t.event_type] || '#888'
                        const isExpanded = expandedTrace === t.id
                        const timeLabel = formatTraceTime(t.timestamp, firstTs)

                        return (
                          <div key={t.id} style={{ marginBottom: 2 }}>
                            <div
                              onClick={() => setExpandedTrace(isExpanded ? null : t.id)}
                              style={{
                                display: 'flex',
                                alignItems: 'center',
                                gap: 8,
                                padding: '4px 8px',
                                borderRadius: 4,
                                background: isExpanded ? '#1e1e1e' : 'transparent',
                                cursor: (t.input_ref || t.output_ref) ? 'pointer' : 'default',
                                fontSize: 12,
                                fontFamily: 'monospace',
                                color: '#ccc',
                                borderLeft: `2px solid ${color}66`,
                              }}
                            >
                              <span style={{ color: '#444', minWidth: 28, fontSize: 10 }}>{timeLabel}</span>
                              <span style={{ color, minWidth: 56, textTransform: 'uppercase', fontSize: 9, fontWeight: 600 }}>
                                {t.event_type}
                              </span>
                              <span style={{ flex: 1, wordBreak: 'break-all' }}>{t.summary}</span>
                              {(t.input_ref || t.output_ref) && (
                                <span style={{ color: '#555', fontSize: 10 }}>{isExpanded ? '▾' : '▸'}</span>
                              )}
                            </div>
                            {isExpanded && (t.input_ref || t.output_ref) && (
                              <div style={{
                                margin: '2px 0 6px 34px',
                                padding: 8,
                                background: '#0a0a0a',
                                borderRadius: 4,
                                border: '1px solid #2a2a2a',
                                fontSize: 11,
                                fontFamily: 'monospace',
                                color: '#888',
                                maxWidth: 600,
                              }}>
                                {t.input_ref && (
                                  <div style={{ marginBottom: 4 }}>
                                    <span style={{ color: '#60a5fa' }}>input:</span>{' '}
                                    <span style={{ color: '#aaa' }}>{formatRefValue(t.input_ref)}</span>
                                  </div>
                                )}
                                {t.output_ref && (
                                  <div>
                                    <span style={{ color: '#4ade80' }}>output:</span>{' '}
                                    <span style={{ color: '#aaa' }}>{formatRefValue(t.output_ref)}</span>
                                  </div>
                                )}
                              </div>
                            )}
                          </div>
                        )
                      })
                    })()}
                    {traces.length === 0 && (
                      <div style={{ fontSize: 12, color: '#666', padding: 8 }}>Waiting for events...</div>
                    )}
                  </div>
                </div>
              )}

              {/* Audit tab */}
              {debugTab === 'audit' && (
                <div style={{ flex: 1, overflowY: 'auto', padding: '8px 12px' }}>
                  <div style={{ fontSize: 11, color: '#666', marginBottom: 6 }}>Audit Log</div>
                  {auditLogs.length > 0 ? auditLogs.map((log, i) => (
                    <div key={log.id} style={{
                      display: 'flex',
                      gap: 8,
                      padding: '4px 0',
                      fontSize: 12,
                      fontFamily: 'monospace',
                      borderBottom: '1px solid #1e1e1e',
                    }}>
                      <span style={{ color: '#555', minWidth: 16 }}>{i + 1}</span>
                      <span style={{ color: '#4ade80', minWidth: 100 }}>{log.action}</span>
                      <span style={{ color: '#888', minWidth: 60, textTransform: 'uppercase', fontSize: 10 }}>{log.approval_status}</span>
                      <span style={{ flex: 1, color: '#ccc', wordBreak: 'break-all' }}>{log.input_summary}</span>
                      <span style={{ color: '#555', fontSize: 11 }}>{new Date(log.timestamp).toLocaleTimeString()}</span>
                    </div>
                  )) : (
                    <div style={{ fontSize: 12, color: '#666', padding: 8 }}>No audit logs yet</div>
                  )}
                </div>
              )}

              {/* Eval tab */}
              {debugTab === 'eval' && (
                <div style={{ flex: 1, overflowY: 'auto', padding: '8px 12px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
                    <div style={{ fontSize: 11, color: '#666' }}>Eval Results</div>
                    <button
                      onClick={handleRunEval}
                      disabled={evalRunning || taskStatus !== 'completed'}
                      style={{
                        background: 'none',
                        border: '1px solid #333',
                        color: '#888',
                        cursor: evalRunning ? 'wait' : taskStatus !== 'completed' ? 'not-allowed' : 'pointer',
                        fontSize: 11,
                        padding: '2px 8px',
                        borderRadius: 4,
                      }}
                    >
                      {evalRunning ? 'Running...' : taskStatus !== 'completed' ? 'Waiting' : 'Run Eval'}
                    </button>
                  </div>
                  {evalResult ? (
                    <>
                      <div style={{
                        display: 'flex', gap: 16, marginBottom: 12,
                        fontSize: 13, fontFamily: 'monospace',
                      }}>
                        <span style={{ color: '#fff' }}>Total: {evalResult.total}</span>
                        <span style={{ color: '#4ade80' }}>Passed: {evalResult.passed}</span>
                        <span style={{ color: '#f87171' }}>Failed: {evalResult.failed}</span>
                      </div>
                      {evalResult.results.map(r => (
                        <div key={r.case_id} style={{
                          background: '#1e1e1e',
                          borderRadius: 6,
                          padding: 10,
                          marginBottom: 8,
                          border: `1px solid ${r.passed ? '#22c55e44' : '#ef444444'}`,
                        }}>
                          <div style={{
                            fontSize: 12,
                            color: r.passed ? '#4ade80' : '#f87171',
                            fontWeight: 600,
                            marginBottom: 4,
                          }}>
                            {r.passed ? 'PASS' : 'FAIL'} — {r.case_id}
                          </div>
                          <div style={{ fontSize: 11, color: '#888', marginBottom: 6 }}>{r.goal}</div>
                          {r.details.map((d, i) => (
                            <div key={i} style={{
                              fontSize: 11,
                              fontFamily: 'monospace',
                              color: d.includes('NOT found') || d.includes('found (bad)') ? '#f87171' : '#4ade80',
                              padding: '1px 0',
                            }}>
                              {d}
                            </div>
                          ))}
                        </div>
                      ))}
                    </>
                  ) : (
                    <div style={{ fontSize: 12, color: '#666', padding: 8 }}>
                      {taskStatus === 'completed'
                        ? 'Click "Run Eval" to evaluate'
                        : 'Eval is available after the workflow completes'}
                    </div>
                  )}
                </div>
              )}

              {/* Panels tab */}
              {debugTab === 'panels' && (() => {
                const regionedPanels: RegionedPanels[] = routePanels(layoutConfig, traces, approvals)
                return <Panels regions={regionedPanels} hasLayout={!!layoutConfig} onPanelAction={handlePanelAction} />
              })()}
            </div>

            {/* Approvals (always visible when pending) */}
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
