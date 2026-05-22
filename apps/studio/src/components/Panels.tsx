import type { PanelMessage, RegionedPanels } from '../lib/panels'

interface PanelsProps {
  regions: RegionedPanels[]
  hasLayout: boolean
}

export default function Panels({ regions, hasLayout }: PanelsProps) {
  if (regions.length === 0 || regions.every(r => r.panels.length === 0)) {
    return (
      <div style={{ fontSize: 12, color: '#666', padding: 8 }}>
        No panels yet. Run a workflow to see status cards, reports, and citations.
      </div>
    )
  }

  if (!hasLayout) {
    // Fallback: flow layout without layout.yaml
    return (
      <div style={{ flex: 1, overflowY: 'auto', padding: '8px 12px', display: 'flex', flexWrap: 'wrap', gap: 8, alignContent: 'flex-start' }}>
        {regions.flatMap(r => r.panels).map((panel, i) => (
          <PanelCard key={i} panel={panel} />
        ))}
      </div>
    )
  }

  // Layout-driven: render regions with positioning
  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflowY: 'auto', minHeight: 0 }}>
      {/* Top row: left + main + right */}
      <div style={{
        display: 'flex',
        gap: 8,
        padding: 8,
        overflow: 'hidden',
        minHeight: 176,
        flexShrink: 0,
      }}>
        {regions.filter(r => r.region === 'left').map(r => (
          <RegionColumn key={r.region} region={r} width={220} />
        ))}
        {regions.filter(r => r.region === 'main').map(r => (
          <RegionColumn key={r.region} region={r} width={0} flexGrow />
        ))}
        {regions.filter(r => r.region === 'right').map(r => (
          <RegionColumn key={r.region} region={r} width={220} />
        ))}
      </div>
      {/* Bottom row */}
      {regions.filter(r => r.region === 'bottom').map(r => (
        <div key={r.region} style={{
          minHeight: 128,
          maxHeight: 180,
          borderTop: '1px solid #2a2a2a',
          padding: 8,
          overflowY: 'auto',
          flexShrink: 0,
        }}>
          <div style={{ fontSize: 10, color: '#666', textTransform: 'uppercase', marginBottom: 6, fontWeight: 600 }}>
            {r.region}
          </div>
          {r.panels.map((panel, i) => (
            <PanelCard key={i} panel={panel} />
          ))}
        </div>
      ))}
      {/* Unknown regions */}
      {regions.filter(r => !['left', 'main', 'right', 'bottom'].includes(r.region)).map(r => (
        <RegionColumn key={r.region} region={r} width={0} flexGrow />
      ))}
    </div>
  )
}

function RegionColumn({ region, width, flexGrow }: { region: RegionedPanels; width: number; flexGrow?: boolean }) {
  return (
    <div style={{
      width: width || undefined,
      flex: flexGrow ? 1 : undefined,
      minWidth: flexGrow ? 0 : width,
      minHeight: 0,
      borderRight: '1px solid #2a2a2a',
      paddingRight: 8,
      overflowY: 'auto',
      display: 'flex',
      flexDirection: 'column',
      gap: 8,
    }}>
      <div style={{ fontSize: 10, color: '#666', textTransform: 'uppercase', fontWeight: 600 }}>
        {region.region}
      </div>
      {region.panels.map((panel, i) => (
        <PanelCard key={i} panel={panel} />
      ))}
    </div>
  )
}

function PanelCard({ panel }: { panel: PanelMessage }) {
  switch (panel.panel_type) {
    case 'status_card':
      return <StatusCardPanel panel={panel} />
    case 'report':
      return <ReportPanel panel={panel} />
    case 'citation':
      return <CitationPanel panel={panel} />
    case 'approval':
      return <ApprovalPanel panel={panel} />
    case 'trace_timeline':
      return <TraceTimelinePanel panel={panel} />
    default:
      return <GenericPanel panel={panel} />
  }
}

const STATUS_COLORS: Record<string, { bg: string; border: string; dot: string }> = {
  healthy: { bg: '#22c55e15', border: '#22c55e44', dot: '#22c55e' },
  ok: { bg: '#22c55e15', border: '#22c55e44', dot: '#22c55e' },
  warning: { bg: '#f59e0b15', border: '#f59e0b44', dot: '#f59e0b' },
  error: { bg: '#ef444415', border: '#ef444444', dot: '#ef4444' },
  critical: { bg: '#ef444415', border: '#ef444444', dot: '#ef4444' },
  degraded: { bg: '#f59e0b15', border: '#f59e0b44', dot: '#f59e0b' },
}

function StatusCardPanel({ panel }: { panel: PanelMessage }) {
  const status = panel.data.status || 'unknown'
  const colors = STATUS_COLORS[status] || { bg: '#88815', border: '#88833', dot: '#888' }

  return (
    <div style={{
      background: colors.bg,
      border: `1px solid ${colors.border}`,
      borderRadius: 8,
      padding: 12,
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
        <div style={{ width: 10, height: 10, borderRadius: '50%', background: colors.dot }} />
        <span style={{ fontSize: 12, fontWeight: 600, color: '#fff' }}>{panel.title}</span>
      </div>
      <div style={{ fontSize: 13, fontWeight: 500, color: colors.dot, textTransform: 'capitalize' }}>
        {status}
      </div>
    </div>
  )
}

function ReportPanel({ panel }: { panel: PanelMessage }) {
  return (
    <div style={{
      background: '#1e1e1e',
      border: '1px solid #3b82f644',
      borderRadius: 8,
      padding: 12,
    }}>
      <div style={{ fontSize: 12, fontWeight: 600, color: '#60a5fa', marginBottom: 8 }}>
        {panel.title}
      </div>
      <div style={{ fontSize: 12, color: '#ccc', lineHeight: 1.5, wordBreak: 'break-word' }}>
        {panel.data.summary || panel.data.raw}
      </div>
    </div>
  )
}

function CitationPanel({ panel }: { panel: PanelMessage }) {
  const citations: Array<{ source_id: string; path: string; title: string }> = panel.data.citations || []
  const recommendation: string = panel.data.recommendation || ''

  return (
    <div style={{
      background: '#1e1e1e',
      border: '1px solid #a78bfa44',
      borderRadius: 8,
      padding: 12,
    }}>
      <div style={{ fontSize: 12, fontWeight: 600, color: '#a78bfa', marginBottom: 8 }}>
        {panel.title}
      </div>
      {citations.length > 0 && (
        <div style={{ marginBottom: 8 }}>
          <div style={{ fontSize: 11, color: '#666', marginBottom: 4 }}>Sources:</div>
          {citations.map((c, i) => (
            <div key={i} style={{ fontSize: 11, fontFamily: 'monospace', color: '#a78bfa', padding: '3px 0', lineHeight: 1.6 }}>
              <span style={{ color: '#666' }}>id:</span>{c.source_id}{' '}
              <span style={{ color: '#666' }}>path:</span>{c.path}{' '}
              <span style={{ color: '#666' }}>title:</span>{c.title}
            </div>
          ))}
        </div>
      )}
      {recommendation && (
        <div style={{
          fontSize: 12, color: '#ccc', borderTop: '1px solid #2a2a2a', paddingTop: 8, lineHeight: 1.5,
        }}>
          {recommendation}
        </div>
      )}
    </div>
  )
}

function ApprovalPanel({ panel }: { panel: PanelMessage }) {
  return (
    <div style={{
      background: '#1e1e1e',
      border: '1px solid #f59e0b44',
      borderRadius: 8,
      padding: 12,
    }}>
      <div style={{ fontSize: 12, fontWeight: 600, color: '#f59e0b', marginBottom: 8 }}>
        {panel.title}
      </div>
      <div style={{ fontSize: 12, color: '#ccc', wordBreak: 'break-all' }}>
        {panel.data.action || ''}
      </div>
    </div>
  )
}

function TraceTimelinePanel({ panel }: { panel: PanelMessage }) {
  const traces: Array<{ event_type: string; summary: string; timestamp: string }> = panel.data.traces || []

  return (
    <div style={{
      background: '#1e1e1e',
      border: '1px solid #88833',
      borderRadius: 8,
      padding: 12,
      maxHeight: 200,
      overflowY: 'auto',
    }}>
      <div style={{ fontSize: 12, fontWeight: 600, color: '#888', marginBottom: 8 }}>
        {panel.title}
      </div>
      {traces.map((t, i) => {
        const color = t.event_type === 'system' ? '#888'
          : t.event_type === 'tool' ? '#60a5fa'
          : t.event_type === 'agent' ? '#a78bfa'
          : t.event_type === 'approval' ? '#f59e0b'
          : '#888'
        return (
          <div key={i} style={{
            display: 'flex', gap: 8, padding: '2px 0', fontSize: 11, fontFamily: 'monospace', color: '#ccc',
          }}>
            <span style={{ color, minWidth: 50, textTransform: 'uppercase', fontSize: 9 }}>{t.event_type}</span>
            <span style={{ flex: 1, wordBreak: 'break-all', color: '#aaa' }}>{t.summary}</span>
          </div>
        )
      })}
    </div>
  )
}

function GenericPanel({ panel }: { panel: PanelMessage }) {
  return (
    <div style={{ background: '#1e1e1e', border: '1px solid #333', borderRadius: 8, padding: 12 }}>
      <div style={{ fontSize: 12, fontWeight: 600, color: '#fff', marginBottom: 8 }}>{panel.title}</div>
      <pre style={{ fontSize: 11, color: '#888', margin: 0, whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
        {JSON.stringify(panel.data, null, 2)}
      </pre>
    </div>
  )
}
