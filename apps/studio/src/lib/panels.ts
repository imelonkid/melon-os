import { parse as parseYaml } from 'yaml'
import type { ApprovalItem, TraceEvent } from './api'

export interface PanelMessage {
  panel_type: 'status_card' | 'report' | 'citation' | 'approval' | 'trace_timeline'
  title: string
  data: Record<string, any>
  actions?: PanelAction[]
}

export interface PanelAction {
  label: string
  action: string
  params?: Record<string, any>
}

export interface CitationEntry {
  source_id: string
  path: string
  title: string
}

/**
 * UI layout config parsed from ui/layout.yaml.
 * Matches crates/melon-scenario/src/ui.rs UiLayout schema.
 */
export interface UiLayoutConfig {
  views: { id: string; view_type: string; region?: string }[]
}

/**
 * Panel grouped by region for layout-driven rendering.
 */
export interface RegionedPanels {
  region: string
  panels: PanelMessage[]
}

type PanelType = PanelMessage['panel_type']

// --- View type -> panel type mapping ---

const VIEW_TYPE_TO_PANEL: Record<string, PanelType> = {
  document: 'report',
  table: 'status_card',
  task_graph: 'trace_timeline',
  device_panel: 'status_card',
  approval: 'approval',
}

/**
 * Map a layout view_type to the panel type it should render.
 */
export function viewTypeToPanelType(viewType: string): PanelType | null {
  return VIEW_TYPE_TO_PANEL[viewType] ?? null
}

// --- Trace marker extraction ---

function extractStatus(summary: string): string {
  const match = summary.match(/status["\s:=]+["']?(healthy|warning|error|ok|critical|degraded)/i)
  if (match) return match[1].toLowerCase()
  if (summary.includes('warning')) return 'warning'
  if (summary.includes('error') || summary.includes('critical')) return 'error'
  if (summary.includes('healthy') || summary.includes('ok')) return 'healthy'
  return 'unknown'
}

/**
 * Extract citation triplets from trace summary.
 * Format: source_id=X path=Y title=Z
 */
export function extractCitations(summary: string): CitationEntry[] {
  const citations = new Map<string, CitationEntry>()
  // Match source_id, path, title - title can contain dots (filenames) but stops at ] or whitespace
  const re = /source_id=(\S+)\s+path=(\S+)\s+title=([^\s\]]+)/g
  let m
  while ((m = re.exec(summary)) !== null) {
    const entry = {
      source_id: m[1],
      path: m[2],
      title: m[3].replace(/[.,;]+$/, ''), // trim trailing punctuation
    }
    citations.set(`${entry.source_id}:${entry.path}:${entry.title}`, entry)
  }
  return Array.from(citations.values())
}

function extractRecommendation(summary: string): string {
  const match = summary.match(/\[actionable_recommendation\]\s*(.*?)(?:\. Review|\.|$)/i)
  return match ? match[1].trim() : ''
}

function extractPlainText(summary: string): string {
  return summary.replace(/\[[\w_]+\]\s*/g, '').trim()
}

function buildStatusPanels(traces: TraceEvent[]): PanelMessage[] {
  const panels: PanelMessage[] = []

  for (const t of traces) {
    const summary = t.summary || ''

    if (summary.includes('[service_status]')) {
      panels.push({
        panel_type: 'status_card',
        title: 'Service Status',
        data: { status: extractStatus(summary), raw: summary },
      })
    }

    if (summary.includes('[storage_status]')) {
      const status = extractStatus(summary)
      panels.push({
        panel_type: 'status_card',
        title: 'Storage Status',
        data: { status, raw: summary, severity: status === 'warning' ? 'warning' : status === 'error' ? 'error' : 'ok' },
      })
    }

    if (summary.includes('[network_status]')) {
      panels.push({
        panel_type: 'status_card',
        title: 'Network Status',
        data: { status: extractStatus(summary), raw: summary },
      })
    }
  }

  return panels
}

function buildCitationPanels(traces: TraceEvent[]): PanelMessage[] {
  const citationMap = new Map<string, CitationEntry>()
  let recommendation = ''
  let raw = ''

  for (const t of traces) {
    const summary = t.summary || ''
    if (!summary.includes('[source_reference]')) continue

    for (const citation of extractCitations(summary)) {
      citationMap.set(`${citation.source_id}:${citation.path}:${citation.title}`, citation)
    }
    recommendation = extractRecommendation(summary) || recommendation
    raw = summary
  }

  const citations = Array.from(citationMap.values())
  if (citations.length === 0 && !recommendation) return []

  return [{
    panel_type: 'citation',
    title: 'Knowledge Sources',
    data: { citations, recommendation, raw },
  }]
}

function buildReportPanels(traces: TraceEvent[]): PanelMessage[] {
  const summaries: string[] = []
  let storageWarning = ''
  let recommendation = ''
  let raw = ''

  for (const t of traces) {
    const summary = t.summary || ''
    if (summary.includes('[inspection_summary]')) {
      summaries.push(extractPlainText(summary))
      raw = summary
    }
    if (summary.includes('[storage_status]')) {
      storageWarning = extractPlainText(summary)
    }
    if (summary.includes('[actionable_recommendation]')) {
      recommendation = extractRecommendation(summary) || extractPlainText(summary)
      raw = summary
    }
  }

  if (summaries.length === 0 && !storageWarning && !recommendation) return []

  return [{
    panel_type: 'report',
    title: 'Inspection Report',
    data: {
      summary: summaries.join('\n'),
      storage_warning: storageWarning,
      recommendation,
      raw,
    },
  }]
}

function buildApprovalPanels(approvals: ApprovalItem[]): PanelMessage[] {
  return approvals.map(approval => ({
    panel_type: 'approval',
    title: 'Approval Request',
    data: {
      id: approval.id,
      action: approval.action,
      risk_level: approval.risk_level,
      scope: approval.scope,
      status: approval.status,
    },
    actions: [
      { label: 'Approve', action: 'approval.approve', params: { approval_id: approval.id } },
      { label: 'Reject', action: 'approval.reject', params: { approval_id: approval.id } },
    ],
  }))
}

function buildTraceTimelinePanel(traces: TraceEvent[]): PanelMessage[] {
  if (traces.length === 0) return []
  return [{
    panel_type: 'trace_timeline',
    title: 'Trace Timeline',
    data: { traces },
  }]
}

// --- Derive panels from traces (layout-agnostic) ---

/**
 * Derive PanelMessage[] from trace events by parsing known trace markers.
 * Used as fallback when no ui/layout.yaml is present.
 */
export function derivePanels(traces: TraceEvent[], approvals: ApprovalItem[] = []): PanelMessage[] {
  const panels: PanelMessage[] = []
  panels.push(...buildStatusPanels(traces))
  panels.push(...buildReportPanels(traces))
  panels.push(...buildCitationPanels(traces))
  panels.push(...buildApprovalPanels(approvals))
  panels.push(...buildTraceTimelinePanel(traces))
  return panels
}

// --- Panel Router: layout-driven ---

/**
 * Collect trace-derived data for a given panel type.
 * This routes trace data into the panels specified by the layout.
 */
function collectForPanel(
  panelType: PanelType,
  traces: TraceEvent[],
  approvals: ApprovalItem[],
): PanelMessage[] {
  switch (panelType) {
    case 'status_card':
      return buildStatusPanels(traces)
    case 'report':
      return buildReportPanels(traces)
    case 'citation':
      return buildCitationPanels(traces)
    case 'approval':
      return buildApprovalPanels(approvals)
    case 'trace_timeline':
      return buildTraceTimelinePanel(traces)
  }
}

/**
 * Panel Router: given a ui/layout.yaml config and trace events,
 * produce panels grouped by region.
 *
 * Each view in the layout is mapped to a panel type.
 * Traces are routed into the corresponding panel, then grouped by region.
 */
export function routePanels(
  layout: UiLayoutConfig | null,
  traces: TraceEvent[],
  approvals: ApprovalItem[] = [],
): RegionedPanels[] {
  if (!layout || layout.views.length === 0) {
    // Fallback: no layout, derive panels as single region
    const derived = derivePanels(traces, approvals)
    return derived.length > 0 ? [{ region: 'main', panels: derived }] : []
  }

  const regionMap = new Map<string, PanelMessage[]>()

  for (const view of layout.views) {
    const region = view.region || 'main'
    const panelType = viewTypeToPanelType(view.view_type)
    if (!panelType) continue

    const collected = view.view_type === 'table'
      ? [
          ...collectForPanel('status_card', traces, approvals),
          ...collectForPanel('citation', traces, approvals),
        ]
      : collectForPanel(panelType, traces, approvals)
    if (collected.length === 0) continue

    if (!regionMap.has(region)) {
      regionMap.set(region, [])
    }
    regionMap.get(region)!.push(...collected)
  }

  // Sort regions in canonical order: left, main, right, bottom
  const regionOrder = ['left', 'main', 'right', 'bottom']
  const result: RegionedPanels[] = []

  for (const region of regionOrder) {
    if (regionMap.has(region)) {
      result.push({ region, panels: regionMap.get(region)! })
    }
  }
  // Add any regions not in canonical order
  for (const [region, panels] of regionMap) {
    if (!regionOrder.includes(region)) {
      result.push({ region, panels })
    }
  }

  return result
}

/**
 * Parse ui/layout.yaml content into UiLayoutConfig.
 */
export function parseLayout(yamlContent: string): UiLayoutConfig | null {
  try {
    const parsed = parseYaml(yamlContent) as { views?: unknown[] } | null
    if (!parsed || !parsed.views || !Array.isArray(parsed.views)) {
      return null
    }
    return {
      views: parsed.views.map((v: any) => ({
        id: String(v.id || ''),
        view_type: String(v.type || v.view_type || ''),
        region: v.region ? String(v.region) : undefined,
      })),
    }
  } catch {
    return null
  }
}
