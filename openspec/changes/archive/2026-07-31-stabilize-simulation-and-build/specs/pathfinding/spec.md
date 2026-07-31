## MODIFIED Requirements

### Requirement: Path component
Each unit entity SHALL have a `Path` component containing world-coordinate waypoints. A newly created unit SHALL have a non-empty path to a reachable walkable tile.

#### Scenario: Units spawn with reachable path
- **WHEN** `createGameWorld(unitCount, seed)` is called
- **THEN** every spawned unit has at least one waypoint
- **THEN** every waypoint is the center of a walkable tile
- **THEN** the path is reachable from the unit's initial tile

### Requirement: find_path_action system
The system SHALL read completed path events, choose reachable walkable targets, and assign only A*-validated paths. It SHALL never assign a direct route that can cross blocked tiles.

#### Scenario: Path found and assigned
- **WHEN** a unit needs a new wandering target
- **THEN** the target is walkable and reachable from the current tile
- **THEN** the assigned waypoints are returned by A* and lie on walkable tile centers

#### Scenario: No reachable target on an attempt
- **WHEN** a candidate target is blocked or unreachable
- **THEN** that candidate is discarded
- **THEN** another candidate is tried
- **THEN** no direct obstacle-crossing fallback path is assigned

#### Scenario: Path respects tile boundaries
- **WHEN** a unit follows a path
- **THEN** the unit never crosses a blocked tile

