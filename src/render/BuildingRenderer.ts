import { Container, Graphics } from 'pixi.js'
import { RENDER_TILE_SIZE } from './SceneRenderer'

export type MapObject = {
  col: number
  row: number
  kind: 'wall' | 'bed' | 'berrybush' | 'ConstructionSite'
  underlying?: 'wall' | 'bed' | 'berrybush'
}

export type ConstructionProgress = {
  col: number
  row: number
  progress: number
  maxProgress: number
  kind: string
}

function parseMapObjects(json: string): MapObject[] {
  return JSON.parse(json) as MapObject[]
}

function parseConstructionProgress(json: string): ConstructionProgress[] {
  return JSON.parse(json) as ConstructionProgress[]
}

export class BuildingRenderer {
  private readonly container: Container
  private sprites = new Map<string, Graphics>()
  private progressBars = new Map<string, Graphics>()

  constructor(container: Container) {
    this.container = container
  }

  sync(mapJson: string, progressJson?: string): void {
    const objects = parseMapObjects(mapJson)
    const seen = new Set<string>()

    const progressMap = new Map<string, { current: number; max: number }>()
    if (progressJson) {
      const progressData = parseConstructionProgress(progressJson)
      for (const p of progressData) {
        progressMap.set(`${p.col},${p.row}`, { current: p.progress, max: p.maxProgress })
      }
    }

    for (const obj of objects) {
      const key = `${obj.col},${obj.row}`
      seen.add(key)

      let g = this.sprites.get(key)
      if (!g) {
        g = new Graphics()
        g.position.set(
          obj.col * RENDER_TILE_SIZE,
          obj.row * RENDER_TILE_SIZE,
        )
        this.sprites.set(key, g)
        this.container.addChild(g)
      }
      this.drawObject(g, obj)
    }

    for (const [key, g] of this.sprites) {
      if (!seen.has(key)) {
        g.removeFromParent()
        this.sprites.delete(key)
      }
    }

    this.syncProgressBars(seen, progressMap)
  }

  private drawObject(g: Graphics, obj: MapObject): void {
    g.clear()
    if (obj.kind === 'ConstructionSite') {
      const kind = obj.underlying ?? 'wall'
      this.drawBuildingShape(g, kind)
      g.alpha = 0.5
    } else {
      g.alpha = 1
      this.drawBuildingShape(g, obj.kind)
    }
  }

  private drawBuildingShape(g: Graphics, kind: string): void {
    switch (kind) {
      case 'wall':
        g.rect(0, 0, RENDER_TILE_SIZE, RENDER_TILE_SIZE)
        g.fill({ color: 0x666666 })
        g.stroke({ color: 0x444444, width: 1 })
        break
      case 'bed':
        g.rect(4, RENDER_TILE_SIZE - 8, RENDER_TILE_SIZE - 8, 6)
        g.fill({ color: 0x8b4513 })
        g.rect(4, RENDER_TILE_SIZE - 14, RENDER_TILE_SIZE - 8, 6)
        g.fill({ color: 0x654321 })
        break
      case 'berrybush': {
        g.moveTo(8, RENDER_TILE_SIZE)
        g.arc(16, RENDER_TILE_SIZE, 8, Math.PI, 0)
        g.closePath()
        g.fill({ color: 0x2d8a4e })
        g.circle(12, RENDER_TILE_SIZE - 8, 3)
        g.fill({ color: 0x7b2d8e })
        g.circle(20, RENDER_TILE_SIZE - 8, 3)
        g.fill({ color: 0x7b2d8e })
        g.circle(16, RENDER_TILE_SIZE - 12, 3)
        g.fill({ color: 0x7b2d8e })
        break
      }
    }
  }

  private syncProgressBars(
    seen: Set<string>,
    progressMap: Map<string, { current: number; max: number }>,
  ): void {
    const barSeen = new Set<string>()

    for (const key of seen) {
      const progress = progressMap.get(key)
      if (progress === undefined) continue

      barSeen.add(key)
      let bar = this.progressBars.get(key)
      if (!bar) {
        bar = new Graphics()
        bar.position.set(
          Number(key.split(',')[0]) * RENDER_TILE_SIZE,
          Number(key.split(',')[1]) * RENDER_TILE_SIZE,
        )
        this.progressBars.set(key, bar)
        this.container.addChild(bar)
      }
      this.drawProgressBar(bar, progress.current, progress.max)
    }

    for (const [key, bar] of this.progressBars) {
      if (!barSeen.has(key)) {
        bar.removeFromParent()
        this.progressBars.delete(key)
      }
    }
  }

  private drawProgressBar(g: Graphics, current: number, max: number): void {
    const barWidth = RENDER_TILE_SIZE - 4
    const barHeight = 3
    const x = 2
    const y = 0
    const ratio = Math.min(current / max, 1)

    g.clear()
    g.rect(x, y, barWidth, barHeight)
    g.fill({ color: 0x333333 })
    g.rect(x, y, barWidth * ratio, barHeight)
    g.fill({ color: 0x00ff00 })
  }

  destroy(): void {
    for (const g of this.sprites.values()) {
      g.removeFromParent()
    }
    this.sprites.clear()
    for (const bar of this.progressBars.values()) {
      bar.removeFromParent()
    }
    this.progressBars.clear()
  }
}
