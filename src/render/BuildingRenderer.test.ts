import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { Container, Graphics } from 'pixi.js'
import { BuildingRenderer, type MapObject, type ConstructionProgress } from './BuildingRenderer'

const RENDER_TILE_SIZE = 32

function makeMapJson(objects: MapObject[]): string {
  return JSON.stringify(objects)
}

function makeProgressJson(data: ConstructionProgress[]): string {
  return JSON.stringify(data)
}

describe('BuildingRenderer', () => {
  let container: Container
  let renderer: BuildingRenderer

  beforeEach(() => {
    container = new Container()
    renderer = new BuildingRenderer(container)
  })

  afterEach(() => {
    renderer.destroy()
  })

  describe('sync', () => {
    it('creates sprites for each map object', () => {
      const json = makeMapJson([
        { col: 0, row: 0, kind: 'wall' },
        { col: 1, row: 2, kind: 'bed' },
      ])
      renderer.sync(json)
      expect(container.children.length).toBe(2)
    })

    it('removes sprites for deleted objects', () => {
      renderer.sync(makeMapJson([
        { col: 0, row: 0, kind: 'wall' },
        { col: 1, row: 1, kind: 'bed' },
      ]))
      expect(container.children.length).toBe(2)

      renderer.sync(makeMapJson([
        { col: 0, row: 0, kind: 'wall' },
      ]))
      expect(container.children.length).toBe(1)
    })

    it('reuses existing sprite for same tile', () => {
      renderer.sync(makeMapJson([{ col: 0, row: 0, kind: 'wall' }]))
      const first = container.children[0]

      renderer.sync(makeMapJson([{ col: 0, row: 0, kind: 'wall' }]))
      expect(container.children.length).toBe(1)
      expect(container.children[0]).toBe(first)
    })

    it('positions sprite at tile coordinates', () => {
      renderer.sync(makeMapJson([{ col: 3, row: 5, kind: 'wall' }]))
      const sprite = container.children[0] as Graphics
      expect(sprite.position.x).toBe(3 * RENDER_TILE_SIZE)
      expect(sprite.position.y).toBe(5 * RENDER_TILE_SIZE)
    })

    it('handles construction sites with underlying type', () => {
      const json = makeMapJson([
        { col: 0, row: 0, kind: 'ConstructionSite', underlying: 'wall' },
      ])
      renderer.sync(json)
      expect(container.children.length).toBe(1)
      const sprite = container.children[0] as Graphics
      expect(sprite.alpha).toBe(0.5)
    })

    it('sets alpha 1 for finished buildings', () => {
      renderer.sync(makeMapJson([{ col: 0, row: 0, kind: 'wall' }]))
      const sprite = container.children[0] as Graphics
      expect(sprite.alpha).toBe(1)
    })
  })

  describe('progress bars', () => {
    it('creates progress bar for construction site with progress data', () => {
      const mapJson = makeMapJson([
        { col: 2, row: 3, kind: 'ConstructionSite', underlying: 'wall' },
      ])
      const progressJson = makeProgressJson([
        { col: 2, row: 3, progress: 25, maxProgress: 50, kind: 'wall' },
      ])
      renderer.sync(mapJson, progressJson)
      expect(container.children.length).toBe(2)
    })

    it('does not create progress bar when no progress data', () => {
      const mapJson = makeMapJson([
        { col: 2, row: 3, kind: 'ConstructionSite', underlying: 'wall' },
      ])
      renderer.sync(mapJson)
      expect(container.children.length).toBe(1)
    })

    it('removes progress bar when construction completes', () => {
      const mapJson = makeMapJson([
        { col: 2, row: 3, kind: 'ConstructionSite', underlying: 'wall' },
      ])
      const progressJson = makeProgressJson([
        { col: 2, row: 3, progress: 25, maxProgress: 50, kind: 'wall' },
      ])
      renderer.sync(mapJson, progressJson)
      expect(container.children.length).toBe(2)

      renderer.sync(makeMapJson([
        { col: 2, row: 3, kind: 'wall' },
      ]))
      expect(container.children.length).toBe(1)
    })

    it('positions progress bar at correct tile', () => {
      const mapJson = makeMapJson([
        { col: 7, row: 4, kind: 'ConstructionSite', underlying: 'berrybush' },
      ])
      const progressJson = makeProgressJson([
        { col: 7, row: 4, progress: 30, maxProgress: 40, kind: 'berrybush' },
      ])
      renderer.sync(mapJson, progressJson)
      const bar = container.children[1] as Graphics
      expect(bar.position.x).toBe(7 * RENDER_TILE_SIZE)
      expect(bar.position.y).toBe(4 * RENDER_TILE_SIZE)
    })

    it('handles multiple tiles with and without progress', () => {
      const mapJson = makeMapJson([
        { col: 0, row: 0, kind: 'wall' },
        { col: 1, row: 0, kind: 'ConstructionSite', underlying: 'bed' },
        { col: 2, row: 0, kind: 'ConstructionSite', underlying: 'berrybush' },
      ])
      const progressJson = makeProgressJson([
        { col: 1, row: 0, progress: 40, maxProgress: 80, kind: 'bed' },
      ])
      renderer.sync(mapJson, progressJson)
      expect(container.children.length).toBe(4)
    })
  })

  describe('destroy', () => {
    it('removes all children from container', () => {
      renderer.sync(makeMapJson([
        { col: 0, row: 0, kind: 'wall' },
        { col: 1, row: 1, kind: 'bed' },
      ]))
      expect(container.children.length).toBe(2)
      renderer.destroy()
      expect(container.children.length).toBe(0)
    })

    it('clears internal maps', () => {
      renderer.sync(makeMapJson([
        { col: 0, row: 0, kind: 'wall' },
      ]))
      renderer.destroy()
      renderer.sync(makeMapJson([
        { col: 0, row: 0, kind: 'wall' },
      ]))
      expect(container.children.length).toBe(1)
    })
  })
})

describe('JSON parsing (integration via sync)', () => {
  let container: Container
  let renderer: BuildingRenderer

  beforeEach(() => {
    container = new Container()
    renderer = new BuildingRenderer(container)
  })

  afterEach(() => {
    renderer.destroy()
  })

  it('parses real WASM-style map objects JSON', () => {
    const json = JSON.stringify([
      { col: 0, row: 0, kind: 'wall' },
      { col: 1, row: 0, kind: 'ConstructionSite', underlying: 'wall' },
    ])
    renderer.sync(json)
    expect(container.children.length).toBe(2)
  })

  it('parses real WASM-style construction progress JSON', () => {
    const mapJson = JSON.stringify([
      { col: 0, row: 0, kind: 'ConstructionSite', underlying: 'bed' },
    ])
    const progressJson = JSON.stringify([
      { col: 0, row: 0, progress: 15, maxProgress: 80, kind: 'bed' },
    ])
    renderer.sync(mapJson, progressJson)
    expect(container.children.length).toBe(2)
  })
})
