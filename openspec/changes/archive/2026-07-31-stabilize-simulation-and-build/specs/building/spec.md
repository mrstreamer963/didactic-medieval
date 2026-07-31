## MODIFIED Requirements

### Requirement: Player can place buildings on the map
When build mode is active and the player clicks a valid tile on the game map, the system SHALL send a `BuildRequest` with the tile coordinates and building type.

#### Scenario: Place building on valid tile
- **WHEN** player clicks a walkable empty tile while in build mode
- **THEN** a `BuildRequest` is sent
- **THEN** a construction site appears on that tile

#### Scenario: Place building on occupied tile
- **WHEN** player clicks a tile that already has a building or construction site
- **THEN** the build request is ignored

#### Scenario: Place any building on blocked tile
- **WHEN** player clicks a blocked non-walkable tile
- **THEN** the build request is ignored for walls, beds, and berry bushes

#### Scenario: Place wall on walkable tile
- **WHEN** player builds a wall on a walkable tile
- **THEN** the tile becomes blocked only after construction completes
- **THEN** future A* paths do not use the completed wall tile

