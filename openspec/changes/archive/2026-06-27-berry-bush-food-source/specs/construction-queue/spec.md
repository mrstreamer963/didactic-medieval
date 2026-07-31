# Construction Queue — Delta

## MODIFIED Requirements

### Requirement: ConstructionQueue resource
The simulation SHALL have a `ConstructionQueue` resource containing all active construction jobs.

#### Scenario: ConstructionQueue structure
- **GIVEN** a construction job exists
- **THEN** it SHALL contain: `col`, `row`, `kind` (Wall/Bed/BerryBush), `progress` (0.0–100.0), `max_progress`, and `assigned_units: Vec<Entity>`

#### Scenario: BuildRequest creates ConstructionJob
- **WHEN** `construction_system` receives a `BuildRequest` for berry bush
- **THEN** it adds a `ConstructionJob` with `kind: BerryBush`, `max_progress: 40.0`, `progress: 0.0` and inserts `ConstructionSite(BerryBush)` into `MapObjects`

#### Scenario: Construction job removal
- **WHEN** a construction job reaches `progress >= max_progress`
- **THEN** the job is removed from the queue; `ConstructionSite` is replaced with `FoodSource(BerryBush, 5)` in `MapObjects`

### Requirement: Different buildings have different build times

#### Scenario: Different buildings have different build times
- **GIVEN** a Wall has `max_progress: 50`, a Bed has `max_progress: 80`, a BerryBush has `max_progress: 40`
- **WHEN** each is built by one colonist
- **THEN** Wall takes ~3.3s, Bed ~5.3s, BerryBush ~2.7s

### Requirement: Berry bush does not block pathfinding
When a berry bush is placed or constructed, the underlying tile SHALL remain walkable.

#### Scenario: Berry bush tile is walkable
- **WHEN** a `FoodSource(BerryBush, _)` or `ConstructionSite(BerryBush)` is on a tile
- **THEN** the tile remains walkable in `TileMapResource`; A* paths are not affected

## REMOVED Requirements

### Requirement: Campfire construction
**Reason**: Campfire replaced by BerryBush
**Migration**: See MODIFIED - BerryBush has max_progress: 40.0

#### Scenario: Campfire build time
**Reason**: Campfire removed
**Migration**: BerryBush has max_progress: 40.0 instead of Campfire's 60.0
