import type { TraceEvent } from './api'

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

// --- View type -> panel type mapping ---

const VIEW_TYPE_TO_PANEL: Record<string, PanelMessage['panel_type']> = {
  document: 'report',
  table: 'status_card',
  task_graph: 'trace_timeline',
  device_panel: 'status_card',
  approval: 'approval',
}

/**
 * Map a layout view_type to the panel type it should render.
 */
export function viewTypeToPanelType(viewType: string): PanelMessage['panel_type'] | null {
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
  const citations: CitationEntry[] = []
  // Match source_id, path, title - title can contain dots (filenames) but stops at ] or whitespace
  const re = /source_id=(\S+)\s+path=(\S+)\s+title=([^\s\]]+)/g
  let m
  while ((m = re.exec(summary)) !== null) {
    citations.push({
      source_id: m[1],
      path: m[2],
      title: m[3].replace(/[.,;]+$/, ''), // trim trailing punctuation
    })
  }
  return citations
}

function extractRecommendation(summary: string): string {
  const match = summary.match(/\[actionable_recommendation\]\s*(.*?)(?:\. Review|\.|$)/i)
  return match ? match[1].trim() : ''
}

function extractPlainText(summary: string): string {
  return summary.replace(/\[[\w_]+\]\s*/g, '').trim()
}

// --- Derive panels from traces (layout-agnostic) ---

/**
 * Derive PanelMessage[] from trace events by parsing known trace markers.
 * Used as fallback when no ui/layout.yaml is present.
 */
export function derivePanels(traces: TraceEvent[]): PanelMessage[] {
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

    if (summary.includes('[source_reference]')) {
      const citations = extractCitations(summary)
      const recommendation = extractRecommendation(summary)
      panels.push({
        panel_type: 'citation',
        title: 'Knowledge Sources',
        data: { citations, recommendation, raw: summary },
      })
    }

    if (summary.includes('[inspection_summary]') || summary.includes('[actionable_recommendation]')) {
      panels.push({
        panel_type: 'report',
        title: 'Inspection Report',
        data: { summary: extractPlainText(summary), raw: summary },
      })
    }

    if (t.event_type === 'approval') {
      panels.push({
        panel_type: 'approval',
        title: 'Approval Request',
        data: { action: summary, input_ref: t.input_ref, output_ref: t.output_ref },
      })
    }
  }

  return panels
}

// --- Panel Router: layout-driven ---

/**
 * Collect trace-derived data for a given panel type.
 * This routes trace data into the panels specified by the layout.
 */
function collectForPanel(
  panelType: PanelMessage['panel_type'],
  traces: TraceEvent[],
): PanelMessage[] {
  const panels: PanelMessage[] = []

  for (const t of traces) {
    const summary = t.summary || ''

    switch (panelType) {
      case 'status_card': {
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
        break
      }
      case 'report': {
        if (summary.includes('[inspection_summary]') || summary.includes('[actionable_recommendation]')) {
          panels.push({
            panel_type: 'report',
            title: 'Inspection Report',
            data: { summary: extractPlainText(summary), raw: summary },
          })
        }
        break
      }
      case 'citation': {
        if (summary.includes('[source_reference]')) {
          const citations = extractCitations(summary)
          const recommendation = extractRecommendation(summary)
          panels.push({
            panel_type: 'citation',
            title: 'Knowledge Sources',
            data: { citations, recommendation, raw: summary },
          })
        }
        break
      }
      case 'trace_timeline': {
        if (panels.length === 0) {
          panels.push({
            panel_type: 'trace_timeline',
            title: 'Trace Timeline',
            data: { traces },
          })
        }
        break
      }
      case 'approval': {
        if (t.event_type === 'approval') {
          panels.push({
            panel_type: 'approval',
            title: 'Approval Request',
            data: { action: summary, input_ref: t.input_ref, output_ref: t.output_ref },
          })
        }
        break
      }
    }
  }

  return panels
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
): RegionedPanels[] {
  if (!layout || layout.views.length === 0) {
    // Fallback: no layout, derive panels as single region
    const derived = derivePanels(traces)
    return derived.length > 0 ? [{ region: 'main', panels: derived }] : []
  }

  const regionMap = new Map<string, PanelMessage[]>()

  for (const view of layout.views) {
    const region = view.region || 'main'
    const panelType = viewTypeToPanelType(view.view_type)
    if (!panelType) continue

    const collected = view.view_type === 'table'
      ? [
          ...collectForPanel('status_card', traces),
          ...collectForPanel('citation', traces),
        ]
      : collectForPanel(panelType, traces)
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
    // Use the global yaml parser (caller must provide parsed object)
    // This is a simple identity - actual parsing done by caller with yaml package
    const parsed = JSON.parse(yamlContent) as { views?: unknown[] }
    if (!parsed.views || !Array.isArray(parsed.views)) {
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
