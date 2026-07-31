import { useEffect, useRef, useState } from 'react'
import { createGameWorld, type GameWorld } from '../pkg/game_core'
import { SceneRenderer, SCENE_WIDTH, SCENE_HEIGHT, RENDER_TILE_SIZE } from './render/SceneRenderer'
import { loadTileTextures } from './render/textures'
import { buildTerrain, type TileMapData } from './render/terrain'
import { UnitSprite } from './render/UnitSprite'
import { BuildingRenderer } from './render/BuildingRenderer'
import { StatusIcons } from './render/StatusIcons'

export type UnitPosition = {
  id: number
  x: number
  y: number
}

export type UnitState = {
  id: number
  satiation: number
  energy: number
  hungry: boolean
  tired: boolean
  needsPlan: string
  speed: number
  assignedJob: { col: number; row: number; kind: string } | null
}

export type BuildMode = 'wall' | 'bed' | 'berrybush' | null

const DEFAULT_UNIT_COUNT = 3
const HIT_RADIUS = 20

function parseTileMap(json: string): TileMapData {
  return JSON.parse(json) as TileMapData
}

function parseUnitPositions(json: string): UnitPosition[] {
  return JSON.parse(json) as UnitPosition[]
}

function parseUnitStates(json: string): UnitState[] {
  return JSON.parse(json) as UnitState[]
}

type UnitsCanvasProps = {
  seed: number
  buildMode: BuildMode
  gameSpeed: number
  onRegenerate: () => void
  selectedUnitId: number | null
  onSelectUnit: (id: number) => void
  onDeselectUnit: () => void
  onStateChange: (states: UnitState[]) => void
}

export function UnitsCanvas({
  seed,
  buildMode,
  gameSpeed,
  onRegenerate,
  selectedUnitId: _selectedUnitId,
  onSelectUnit,
  onDeselectUnit,
  onStateChange,
}: UnitsCanvasProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const buildModeRef = useRef<BuildMode>(null)
  const gameSpeedRef = useRef(gameSpeed)
  buildModeRef.current = buildMode
  gameSpeedRef.current = gameSpeed
  const [hoverCol, setHoverCol] = useState<number | null>(null)
  const [hoverRow, setHoverRow] = useState<number | null>(null)

  useEffect(() => {
    const parent = containerRef.current
    if (!parent) return

    let cancelled = false
    let scene: SceneRenderer | null = null
    let world: GameWorld | null = null
    let units: UnitSprite[] = []
    let buildings: BuildingRenderer | null = null
    let statusIcons: StatusIcons | null = null

    ;(async () => {
      const renderer = new SceneRenderer()
      await renderer.init(parent)
      if (cancelled) {
        renderer.destroy()
        return
      }
      scene = renderer

      const tileTextures = await loadTileTextures()
      if (cancelled) {
        renderer.destroy()
        return
      }

      world = createGameWorld(DEFAULT_UNIT_COUNT, BigInt(seed))
      const tileMap = parseTileMap(world.getTileMap())
      const terrain = buildTerrain(tileMap, seed, tileTextures)
      renderer.layers.terrain.addChild(terrain)

      for (let i = 0; i < DEFAULT_UNIT_COUNT; i++) {
        units.push(new UnitSprite(i, renderer.layers.units, renderer.layers.decals))
      }

      buildings = new BuildingRenderer(renderer.layers.buildings)
      statusIcons = new StatusIcons(renderer.layers.overlay)
      ;(window as any).__world = world
      ;(window as any).__scene = renderer

      const canvas = renderer.app.canvas

      const handleClick = (e: MouseEvent) => {
        if (!world) return
        const rect = canvas.getBoundingClientRect()
        const scaleX = SCENE_WIDTH / rect.width
        const scaleY = SCENE_HEIGHT / rect.height
        const mx = (e.clientX - rect.left) * scaleX
        const my = (e.clientY - rect.top) * scaleY

        if (buildModeRef.current) {
          const col = Math.floor(mx / RENDER_TILE_SIZE)
          const row = Math.floor(my / RENDER_TILE_SIZE)
          if (col >= 0 && col < 25 && row >= 0 && row < 19) {
            world.build(col, row, buildModeRef.current)
          }
          return
        }

        const worldX = mx / RENDER_TILE_SIZE
        const worldY = my / RENDER_TILE_SIZE
        const positions = parseUnitPositions(world.getUnitPositions())
        let closestId: number | null = null
        let closestDist = HIT_RADIUS * HIT_RADIUS

        for (const pos of positions) {
          const dx = worldX - pos.x
          const dy = worldY - pos.y
          const dist = dx * dx + dy * dy
          if (dist < closestDist) {
            closestDist = dist
            closestId = pos.id
          }
        }

        if (closestId !== null) {
          onSelectUnit(closestId)
        } else {
          onDeselectUnit()
        }
      }

      const handleRightClick = (e: MouseEvent) => {
        e.preventDefault()
      }

      canvas.addEventListener('click', handleClick)
      canvas.addEventListener('contextmenu', handleRightClick)

      renderer.app.ticker.add((ticker) => {
        if (!world) return
        if (gameSpeedRef.current === 0) {
          world.tick(0)
          if (buildings) {
            buildings.sync(world.getMapObjects(), world.getConstructionProgress())
          }
          return
        }
        world.tick(ticker.deltaMS * gameSpeedRef.current)
        const positions = parseUnitPositions(world.getUnitPositions())
        for (const pos of positions) {
          const unit = units[pos.id]
          if (unit) {
            unit.update(pos.x, pos.y, ticker.deltaMS)
          }
        }
        if (buildings) {
          buildings.sync(world.getMapObjects(), world.getConstructionProgress())
        }
        if (statusIcons) {
          const statesJson = world.getUnitStates()
          statusIcons.sync(statesJson, positions)
          onStateChange(parseUnitStates(statesJson))
        }
      })
    })()

    return () => {
      cancelled = true
      for (const u of units) u.destroy()
      units = []
      buildings?.destroy()
      statusIcons?.destroy()
      world?.free()
      world = null
      scene?.destroy()
      scene = null
      while (parent.firstChild) parent.removeChild(parent.firstChild)
    }
  }, [seed])

  useEffect(() => {
    const parent = containerRef.current
    if (!parent) return
    const canvas = parent.querySelector('canvas')
    if (!canvas) return

    const handleMove = (e: MouseEvent) => {
      const rect = canvas.getBoundingClientRect()
      const scaleX = SCENE_WIDTH / rect.width
      const scaleY = SCENE_HEIGHT / rect.height
      const mx = (e.clientX - rect.left) * scaleX
      const my = (e.clientY - rect.top) * scaleY
      const col = Math.floor(mx / RENDER_TILE_SIZE)
      const row = Math.floor(my / RENDER_TILE_SIZE)
      if (col >= 0 && col < 25 && row >= 0 && row < 19) {
        setHoverCol(col)
        setHoverRow(row)
      } else {
        setHoverCol(null)
        setHoverRow(null)
      }
    }

    canvas.addEventListener('mousemove', handleMove)
    return () => canvas.removeEventListener('mousemove', handleMove)
  }, [])

  return (
    <section className="units-section">
      <div
        ref={containerRef}
        className="units-canvas"
        style={{ width: SCENE_WIDTH, height: SCENE_HEIGHT }}
      />
      {buildMode && hoverCol !== null && hoverRow !== null && (
        <div className="build-hint">
          Строительство: {buildMode === 'wall' ? 'Стена' : buildMode === 'bed' ? 'Кровать' : 'Куст'} ({hoverCol}, {hoverRow})
        </div>
      )}
      <button type="button" className="regenerate-btn" onClick={onRegenerate}>
        Перегенерировать
      </button>
    </section>
  )
}
