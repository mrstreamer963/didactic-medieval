import type { UnitState } from './UnitsCanvas'

type UnitPanelProps = {
  unitId: number
  unitStates: UnitState[]
  onClose: () => void
}

export function UnitPanel({ unitId, unitStates, onClose }: UnitPanelProps) {
  const unit = unitStates.find(s => s.id === unitId)
  if (!unit) return null

  const statusParts: string[] = []
  if (unit.hungry) statusParts.push('голоден')
  if (unit.tired) statusParts.push('устал')
  if (unit.assignedJob) {
    const kindNames: Record<string, string> = { wall: 'стену', bed: 'кровать', berrybush: 'куст' }
    statusParts.push(`строит ${kindNames[unit.assignedJob.kind] || unit.assignedJob.kind}`)
  } else if (unit.needsPlan === 'eat') {
    statusParts.push('ищет еду')
  } else if (unit.needsPlan === 'sleep') {
    statusParts.push('ищет сон')
  } else {
    statusParts.push('блуждает')
  }

  const goalText = unit.assignedJob
    ? `Стройка (${unit.assignedJob.col}, ${unit.assignedJob.row})`
    : unit.needsPlan
      ? unit.needsPlan === 'eat' ? 'Куст' : 'Кровать'
      : '—'

  return (
    <div className="unit-panel">
      <button className="unit-panel-close" onClick={onClose}>×</button>
      <div className="unit-panel-content">
        <div className="unit-panel-id">Юнит #{unit.id}</div>

        <div className="unit-panel-row">
          <span className="unit-panel-label">Сытость</span>
          <div className="unit-panel-bar">
            <div
              className="unit-panel-fill unit-panel-fill-food"
              style={{ width: `${unit.satiation}%` }}
            />
          </div>
          <span className="unit-panel-value">{Math.round(unit.satiation)}%</span>
        </div>

        <div className="unit-panel-row">
          <span className="unit-panel-label">Энергия</span>
          <div className="unit-panel-bar">
            <div
              className="unit-panel-fill unit-panel-fill-energy"
              style={{ width: `${unit.energy}%` }}
            />
          </div>
          <span className="unit-panel-value">{Math.round(unit.energy)}%</span>
        </div>

        <div className="unit-panel-info">
          <span className="unit-panel-label">Статус</span>
          <span>{statusParts.join(', ')}</span>
        </div>

        <div className="unit-panel-info">
          <span className="unit-panel-label">Цель</span>
          <span>{goalText}</span>
        </div>

        <div className="unit-panel-info">
          <span className="unit-panel-label">Скорость</span>
          <span>{unit.speed}</span>
        </div>
      </div>
    </div>
  )
}
