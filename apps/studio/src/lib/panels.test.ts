import { describe, it, expect } from 'vitest'
import {
  extractCitations,
  routePanels,
  derivePanels,
  parseLayout,
  viewTypeToPanelType,
  type UiLayoutConfig,
} from './panels'
import type { TraceEvent } from './api'

// --- extractCitations tests ---

describe('extractCitations', () => {
  it('extracts full citation triplets', () => {
    const summary =
      '[source_reference] Retrieved 2 knowledge hit(s): ' +
      'citations=[source_id=inspection_runbook path=knowledge/fixtures/inspection_runbook.md title=inspection_runbook.md]. ' +
      '[actionable_recommendation] Based on sourced runbook excerpt.'
    const citations = extractCitations(summary)
    expect(citations).toHaveLength(1)
    expect(citations[0]).toEqual({
      source_id: 'inspection_runbook',
      path: 'knowledge/fixtures/inspection_runbook.md',
      title: 'inspection_runbook.md',
    })
  })

  it('extracts multiple citations', () => {
    const summary =
      'source_id=a path=docs/a.md title=A.md; source_id=b path=docs/b.md title=B.md source_id=a path=docs/a.md title=A.md'
    const citations = extractCitations(summary)
    expect(citations).toHaveLength(2)
    expect(citations[0].source_id).toBe('a')
    expect(citations[1].source_id).toBe('b')
  })

  it('returns empty array when no citations', () => {
    expect(extractCitations('no citations here')).toEqual([])
  })
})

// --- viewTypeToPanelType tests ---

describe('viewTypeToPanelType', () => {
  it('maps document -> report', () => {
    expect(viewTypeToPanelType('document')).toBe('report')
  })

  it('maps table -> status_card', () => {
    expect(viewTypeToPanelType('table')).toBe('status_card')
  })

  it('maps task_graph -> trace_timeline', () => {
    expect(viewTypeToPanelType('task_graph')).toBe('trace_timeline')
  })

  it('maps device_panel -> status_card', () => {
    expect(viewTypeToPanelType('device_panel')).toBe('status_card')
  })

  it('returns null for unknown types', () => {
    expect(viewTypeToPanelType('chat')).toBeNull()
    expect(viewTypeToPanelType('kanban')).toBeNull()
  })
})

// --- routePanels tests ---

function makeTrace(summary: string, eventType = 'tool'): TraceEvent {
  return {
    id: `t-${Math.random()}`,
    event_type: eventType,
    summary,
    timestamp: new Date().toISOString(),
  }
}

const demoOpsLayout: UiLayoutConfig = {
  views: [
    { id: 'report', view_type: 'document', region: 'main' },
    { id: 'status', view_type: 'table', region: 'right' },
    { id: 'trace', view_type: 'task_graph', region: 'bottom' },
  ],
}

describe('routePanels', () => {
  it('parses real ui/layout.yaml content', () => {
    const layout = parseLayout(`
layout:
  default: ops_workspace
views:
  - id: report
    type: document
    region: main
  - id: status
    type: table
    region: right
  - id: trace
    type: task_graph
    region: bottom
`)

    expect(layout).not.toBeNull()
    expect(layout!.views).toHaveLength(3)
    expect(layout!.views[0]).toEqual({ id: 'report', view_type: 'document', region: 'main' })
  })

  it('groups panels by region from layout', () => {
    const traces = [
      makeTrace('[storage_status] status: warning'),
      makeTrace('[inspection_summary] Generated checklist'),
      makeTrace('[source_reference] source_id=a path=b.md title=B'),
    ]

    const regions = routePanels(demoOpsLayout, traces)

    // Should have main, right, bottom regions
    expect(regions.length).toBeGreaterThanOrEqual(2)

    const rightRegion = regions.find(r => r.region === 'right')
    expect(rightRegion).toBeDefined()
    expect(rightRegion!.panels.some(p => p.panel_type === 'status_card')).toBe(true)
    expect(rightRegion!.panels.some(p => p.panel_type === 'citation')).toBe(true)

    const mainRegion = regions.find(r => r.region === 'main')
    expect(mainRegion).toBeDefined()
    expect(mainRegion!.panels.some(p => p.panel_type === 'report')).toBe(true)
  })

  it('removes status cards when table view is removed from layout', () => {
    const noTableLayout: UiLayoutConfig = {
      views: [
        { id: 'report', view_type: 'document', region: 'main' },
        { id: 'trace', view_type: 'task_graph', region: 'bottom' },
      ],
    }

    const traces = [makeTrace('[storage_status] status: warning')]
    const regions = routePanels(noTableLayout, traces)

    // No region should have status_card since no table view
    const hasStatusCard = regions.some(r =>
      r.panels.some(p => p.panel_type === 'status_card'),
    )
    expect(hasStatusCard).toBe(false)
  })

  it('falls back to derivePanels when layout is null', () => {
    const traces = [
      makeTrace('[storage_status] status: warning'),
      makeTrace('[inspection_summary] Analysis complete'),
    ]

    const regions = routePanels(null, traces)

    expect(regions).toHaveLength(1)
    expect(regions[0].region).toBe('main')
    expect(regions[0].panels.length).toBe(3)
    expect(regions[0].panels[0].panel_type).toBe('status_card')
    expect(regions[0].panels[1].panel_type).toBe('report')
    expect(regions[0].panels[2].panel_type).toBe('trace_timeline')
  })

  it('produces empty panels for empty traces', () => {
    const regions = routePanels(demoOpsLayout, [])
    expect(regions).toHaveLength(0)
  })

  it('includes trace_timeline panel for task_graph view', () => {
    const traces = [
      makeTrace('Step 1: check', 'system'),
      makeTrace('Step 2: check', 'system'),
    ]
    const regions = routePanels(demoOpsLayout, traces)

    const bottomRegion = regions.find(r => r.region === 'bottom')
    expect(bottomRegion).toBeDefined()
    expect(bottomRegion!.panels.some(p => p.panel_type === 'trace_timeline')).toBe(true)
    expect(bottomRegion!.panels.filter(p => p.panel_type === 'trace_timeline')).toHaveLength(1)
  })

  it('routes pending approvals from approval API data', () => {
    const layout: UiLayoutConfig = {
      views: [{ id: 'approval', view_type: 'approval', region: 'right' }],
    }
    const regions = routePanels(layout, [], [{
      id: 'approval-1',
      action: 'cleanup_temp_files',
      risk_level: 'medium',
      scope: 'workspace',
      status: 'pending',
    }])

    const rightRegion = regions.find(r => r.region === 'right')
    expect(rightRegion).toBeDefined()
    expect(rightRegion!.panels).toHaveLength(1)
    expect(rightRegion!.panels[0].panel_type).toBe('approval')
    expect(rightRegion!.panels[0].data.action).toBe('cleanup_temp_files')
  })
})

// --- derivePanels tests (fallback behavior) ---

describe('derivePanels', () => {
  it('derives status_card from storage_status marker', () => {
    const panels = derivePanels([makeTrace('[storage_status] status: warning')])
    expect(panels.some(p => p.panel_type === 'status_card')).toBe(true)
    expect(panels[0].panel_type).toBe('status_card')
    expect(panels[0].data.status).toBe('warning')
  })

  it('derives report from inspection_summary marker', () => {
    const panels = derivePanels([makeTrace('[inspection_summary] Generated checklist')])
    expect(panels.some(p => p.panel_type === 'report')).toBe(true)
  })

  it('derives citation with full source info', () => {
    const summary =
      '[source_reference] source_id=runbook path=docs/runbook.md title=runbook.md'
    const panels = derivePanels([makeTrace(summary)])
    const citationPanel = panels.find(p => p.panel_type === 'citation')
    expect(citationPanel).toBeDefined()
    const citations = citationPanel!.data.citations
    expect(citations).toHaveLength(1)
    expect(citations[0].source_id).toBe('runbook')
    expect(citations[0].path).toBe('docs/runbook.md')
    expect(citations[0].title).toBe('runbook.md')
  })

  it('derives approval from approval event type', () => {
    const panels = derivePanels([], [{
      id: 'approval-1',
      action: 'cleanup_temp_files',
      risk_level: 'medium',
      scope: 'workspace',
      status: 'pending',
    }])
    expect(panels.some(p => p.panel_type === 'approval')).toBe(true)
  })
})
