# pathfinding Specification

## Purpose

A* pathfinding on a tile grid (Manhattan distance, 4-connected), `Path` component replacing `Target`, and ECS systems for path following and goal-seeking with retargeting.

## Requirements

### Requirement: A* pathfinding

The crate SHALL provide a public function `astar(start: (u32, u32), goal: (u32, u32), tile_map: &TileMapResource) -> Option<Vec<(u32, u32)>>` that finds a path on the tile grid using the A* algorithm with Manhattan distance heuristic on a 4-connected grid.

#### Scenario: Path found between two walkable tiles

- **WHEN** `astar` is called with start and goal both on walkable tiles
- **THEN** the result is `Some(path)` where path is a non-empty vector of tile coordinates
- **THEN** `path[0] == start` and `path[path.len()-1] == goal`
- **THEN** each step in the path is between adjacent tiles (up/down/left/right, not diagonal)
- **THEN** all tiles in the path are walkable

#### Scenario: No path to blocked or isolated goal

- **WHEN** `astar` is called with a goal on a blocked tile
- **THEN** the result is `None`
- **WHEN** `astar` is called and all routes to the goal are blocked
- **THEN** the result is `None`

#### Scenario: Start equals goal

- **WHEN** `astar` is called with start equal to goal on a walkable tile
- **THEN** the result is `Some(vec![start])`

### Requirement: Path component

Each unit entity SHALL have a `Path` component instead of the previous `Target` component. `Path` SHALL contain a `waypoints` field: `Vec<(f32, f32)>` of world coordinates in tile-units.

#### Scenario: Units spawn with path

- **WHEN** `createGameWorld(unitCount, seed)` is called
- **THEN** each spawned unit entity has `Position`, `Path`, and `Speed` components
- **THEN** each unit has a non-empty `Path` with at least one waypoint at the center of a reachable walkable tile

### Requirement: move_along_path system

The ECS world SHALL include a `move_along_path` system that moves each unit's `Position` toward the first waypoint in its `Path` at the rate defined by `Speed`. When a unit reaches a waypoint (within arrival threshold), the waypoint is removed. When the last waypoint is reached, a `TargetReached` message is sent.

#### Scenario: Unit moves toward waypoint

- **WHEN** a unit's `Path` has one or more waypoints and `tick(deltaMs)` is called
- **THEN** the unit's `Position` moves closer to the first waypoint
- **THEN** the distance to the first waypoint decreases

#### Scenario: Unit consumes waypoints on arrival

- **WHEN** a unit's `Position` is within arrival threshold of the first waypoint during a tick
- **THEN** that waypoint is removed from the path
- **THEN** the unit begins moving toward the next waypoint

#### Scenario: Unit sends TargetReached on path completion

- **WHEN** a unit reaches its last waypoint and its `Path` becomes empty
- **THEN** a `TargetReached` message is sent for that entity

### Requirement: find_path_action system

The ECS world SHALL include a `find_path_action` system that reads unread `TargetReached` messages, generates a new random target tile within the field, runs A* from the unit's current tile to the target tile, and writes only the resulting world-coordinate waypoints into the unit's `Path` component. It SHALL not assign a direct fallback route when A* fails.

#### Scenario: Path found and assigned

- **WHEN** a `TargetReached` message is sent for a unit on a walkable tile
- **AND** A* finds a path to the random target tile
- **THEN** the unit's `Path` is set to a non-empty sequence of waypoints in world coordinates
- **THEN** the first waypoint is the center of the unit's current tile (or the next tile if already centered)

#### Scenario: Retarget on blocked or unreachable goal

- **WHEN** a `TargetReached` message is sent and A* cannot find a path to the generated target
- **THEN** `find_path_action` generates a new random target and retries A*
- **THEN** this retries until a reachable target is found or leaves the path empty for a later attempt

#### Scenario: Path respects tile boundaries

- **WHEN** a unit follows its path
- **THEN** all waypoints in the path are at the centers of walkable tiles
- **THEN** the unit never crosses a blocked tile
