# Construction Queue

**Purpose**: Buildings are not placed instantly. When the player places a building, a construction site appears on the map and colonists autonomously build it over time.

## Requirements

### Requirement: Player places a construction site
When the player clicks a valid tile in build mode, a `BuildRequest` is sent. Instead of creating the finished building, the system SHALL create a construction site at that tile.

#### Scenario: Place construction site on valid tile
- **WHEN** player clicks a walkable empty tile while in build mode
- **THEN** a `BuildRequest` is sent; a `ConstructionSite(kind)` object appears on that tile in `MapObjects`; a `ConstructionJob` is added to `ConstructionQueue`

#### Scenario: Place construction site on occupied tile
- **WHEN** player clicks a tile that already has a building or construction site
- **THEN** the build request is ignored

#### Scenario: Place construction site on blocked tile
- **WHEN** player clicks a blocked (non-walkable) tile
- **THEN** the build request is ignored

### Requirement: ConstructionQueue resource
The simulation SHALL have a `ConstructionQueue` resource containing all active construction jobs.

#### Scenario: ConstructionQueue structure
- **GIVEN** a construction job exists
- **THEN** it SHALL contain: `col`, `row`, `kind` (Wall/Bed/BerryBush), `progress` (0.0–100.0), `max_progress`, and `assigned_units: Vec<Entity>`

#### Scenario: BuildRequest creates ConstructionJob
- **WHEN** `construction_system` receives a `BuildRequest`
- **THEN** it adds a `ConstructionJob` with `kind`, `progress: 0.0`, and inserts `ConstructionSite(kind)` into `MapObjects`

#### Scenario: Construction job removal
- **WHEN** a construction job reaches `progress >= max_progress`
- **THEN** the job is removed from the queue; `ConstructionSite` is replaced with the finished building in `MapObjects`; for BerryBush, the site becomes `FoodSource(BerryBush, 5)`

### Requirement: Colonists autonomously take construction jobs
Colonists SHALL pick up construction jobs when they are not fulfilling needs (hunger/sleep).

#### Scenario: Free colonist takes nearest job
- **WHEN** a colonist has no `NeedsPlan` and no `AssignedJob`
- **THEN** the `job_assignment_system` finds the nearest `ConstructionJob` with room for workers (`assigned_units.len() < max_workers`), adds the colonist to `assigned_units`, and gives them `AssignedJob`

#### Scenario: Colonist moves to construction site
- **WHEN** a colonist has `AssignedJob`
- **THEN** the system calculates an A* path to the construction site tile; the colonist moves there normally

#### Scenario: Multiple colonists can work on one site
- **WHEN** a construction job has fewer than `max_workers` assigned
- **THEN** additional free colonists may take the same job, accelerating progress

#### Scenario: Colonist releases job for needs
- **WHEN** a colonist with `AssignedJob` receives a `NeedsPlan` (hungry/tired)
- **THEN** the colonist is removed from `assigned_units`; `AssignedJob` is set to `None`; the job remains available for others

#### Scenario: Colonist resumes construction after needs
- **WHEN** a colonist's `NeedsPlan` is cleared (sated/rested)
- **THEN** they may take a construction job again on the next `job_assignment_system` tick

### Requirement: Construction progress accumulates over time
Each tick, active construction jobs gain progress proportional to the number of assigned workers.

#### Scenario: Progress increases with workers
- **WHEN** one colonist is at the construction site for 1 second
- **THEN** progress increases by `build_speed` (e.g., 15/sec)

#### Scenario: More workers, faster progress
- **WHEN** two colonists are assigned to the same site
- **THEN** progress increases at `build_speed * 2` per second

#### Scenario: Building completes
- **WHEN** `progress >= max_progress`
- **THEN** the construction site is replaced with the finished building in `MapObjects`; walls affect pathfinding

#### Scenario: Different buildings have different build times
- **GIVEN** a Wall has `max_progress: 50`, a Bed has `max_progress: 80`, a BerryBush has `max_progress: 40`
- **WHEN** each is built by one colonist
- **THEN** Wall takes ~3.3s, Bed ~5.3s, BerryBush ~2.7s

### Requirement: Construction sites are rendered on the map
The frontend SHALL render construction sites visually distinct from finished buildings, with a progress indicator.

#### Scenario: Construction site renders as incomplete building
- **WHEN** `MapObjects` contains a `ConstructionSite(Wall)`
- **THEN** a semi-transparent or outlined version of the wall sprite is rendered at that tile

#### Scenario: Progress bar shown above construction site
- **WHEN** a construction site has progress > 0
- **THEN** a progress bar overlay is rendered above the tile, reflecting `progress / max_progress`

#### Scenario: Progress bar updates each frame
- **WHEN** the simulation ticks and progress changes
- **THEN** the progress bar visually updates to match

#### Scenario: Construction site replaced by finished building
- **WHEN** a construction job completes
- **THEN** the construction site sprite is removed; the finished building sprite appears in its place

### Requirement: Berry bush does not block pathfinding
When a berry bush is placed or constructed, the underlying tile SHALL remain walkable.

#### Scenario: Berry bush tile is walkable
- **WHEN** a `FoodSource(BerryBush, _)` or `ConstructionSite(BerryBush)` is on a tile
- **THEN** the tile remains walkable in `TileMapResource`; A* paths are not affected

### Requirement: WASM API exposes construction data
The WASM bridge SHALL expose construction queue data to the frontend.

#### Scenario: Get construction progress
- **WHEN** `world.getConstructionProgress()` is called from JS
- **THEN** a JSON array `[{col, row, progress, kind}]` is returned for all active construction sites

#### Scenario: Map objects includes construction sites
- **WHEN** `world.getMapObjects()` is called
- **THEN** construction sites are included as `{kind: "ConstructionSite", underlying: "Wall"}` in the JSON array
