## MODIFIED Requirements

### Requirement: Player places a construction site
When build mode is active and the player clicks a valid tile, a `BuildRequest` SHALL create a construction site and a construction job only if the tile is walkable, inside the map, and empty.

#### Scenario: Place construction site on valid tile
- **WHEN** a build request targets a walkable empty tile
- **THEN** a `ConstructionSite(kind)` object appears in `MapObjects`
- **THEN** a `ConstructionJob` is added with progress `0.0`

#### Scenario: Place construction site on occupied tile
- **WHEN** a build request targets a tile containing any object or construction site
- **THEN** no object or job is created

#### Scenario: Place construction site on blocked tile
- **WHEN** a build request targets a blocked tile
- **THEN** no object or job is created for any building kind

### Requirement: Berry bush does not block pathfinding
When a berry bush is placed or constructed, its tile SHALL remain walkable.

#### Scenario: Berry bush tile is walkable
- **WHEN** a berry bush construction completes
- **THEN** the tile remains walkable in `TileMapResource`
- **THEN** A* paths may use the tile

