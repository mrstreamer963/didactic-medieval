import { describe, expect, it } from 'vitest'
import { getSimulationDelta } from './simulation'

describe('getSimulationDelta', () => {
  it('does not advance the simulation while paused', () => {
    let tickCalls = 0
    const delta = getSimulationDelta(16, 0)
    if (delta !== null) tickCalls++

    expect(delta).toBeNull()
    expect(tickCalls).toBe(0)
  })

  it('scales the ticker delta at active speeds', () => {
    expect(getSimulationDelta(16, 5)).toBe(80)
  })
})
