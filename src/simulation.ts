export function getSimulationDelta(deltaMs: number, gameSpeed: number): number | null {
  if (gameSpeed === 0) return null
  return deltaMs * gameSpeed
}
