const RUNTIME_URL = '' // relative, proxied by Vite

export interface PackSummary {
  id: string
  name: string
  version: string
  description?: string
  path: string
  status?: string
}

export interface TraceEvent {
  id: string
  event_type: string
  summary: string
  input_ref?: string
  output_ref?: string
  timestamp: string
}

export interface ApprovalItem {
  id: string
  action: string
  risk_level: string
  scope: string
  status: string
}

export interface RunResponse {
  task_id: string
  scenario_id: string
  status: string
  user_goal: string
}

export interface TaskDetail {
  id: string
  scenario_id: string
  user_goal: string
  status: string
}

export async function fetchPacks(): Promise<PackSummary[]> {
  const res = await fetch(`${RUNTIME_URL}/api/packs`)
  if (!res.ok) throw new Error(`Failed to fetch packs: ${res.status}`)
  return res.json()
}

export async function createTask(scenarioId: string, userGoal: string) {
  const res = await fetch(`${RUNTIME_URL}/api/tasks`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ scenario_id: scenarioId, user_goal: userGoal }),
  })
  if (!res.ok) throw new Error(`Failed to create task: ${res.status}`)
  return res.json()
}

export async function runPack(packId: string, userGoal: string): Promise<RunResponse> {
  const res = await fetch(`${RUNTIME_URL}/api/packs/${packId}/run`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ user_goal: userGoal }),
  })
  if (!res.ok) throw new Error(`Failed to run pack: ${res.status}`)
  return res.json()
}

export async function getTask(taskId: string): Promise<TaskDetail> {
  const res = await fetch(`${RUNTIME_URL}/api/tasks/${taskId}`)
  if (!res.ok) throw new Error(`Failed to get task: ${res.status}`)
  return res.json()
}

export async function getTraces(taskId: string): Promise<TraceEvent[]> {
  const res = await fetch(`${RUNTIME_URL}/api/tasks/${taskId}/traces`)
  if (!res.ok) throw new Error(`Failed to get traces: ${res.status}`)
  return res.json()
}

export async function getApprovals(taskId: string): Promise<ApprovalItem[]> {
  const res = await fetch(`${RUNTIME_URL}/api/tasks/${taskId}/approvals`)
  if (!res.ok) throw new Error(`Failed to get approvals: ${res.status}`)
  return res.json()
}

export async function resolveApproval(taskId: string, approvalId: string, approve: boolean) {
  const res = await fetch(`${RUNTIME_URL}/api/tasks/${taskId}/approvals/${approvalId}/action`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ action: approve ? 'approve' : 'reject' }),
  })
  if (!res.ok) throw new Error(`Failed to resolve approval: ${res.status}`)
  return res.json()
}

export async function checkHealth(): Promise<{ status: string; version: string }> {
  try {
    const res = await fetch(`${RUNTIME_URL}/api/health`)
    if (!res.ok) return { status: 'error', version: '' }
    return res.json()
  } catch {
    return { status: 'disconnected', version: '' }
  }
}
