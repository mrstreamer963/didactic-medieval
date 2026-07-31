# rust-wasm-core Specification

## Purpose

Rust/WASM-модуль `crates/game_core`: Cargo workspace, сборка в `pkg/`, API `getProgramName()` для связки TypeScript ↔ Rust.

## Requirements

### Requirement: Cargo workspace

The repository SHALL define a Cargo workspace at the repository root with `crates/game_core` as a workspace member.

#### Scenario: Workspace manifest exists

- **WHEN** a developer inspects the repository root
- **THEN** a `Cargo.toml` file exists with `[workspace]` listing `crates/game_core` as a member

### Requirement: WASM crate build

The `crates/game_core` crate SHALL compile to WebAssembly for the `wasm32-unknown-unknown` target and produce loadable artifacts in a `pkg/` directory at the repository root via `wasm-pack`.

#### Scenario: WASM build succeeds

- **WHEN** the user runs the project's WASM build script (`build:wasm` or equivalent documented in `package.json`)
- **THEN** the command completes with exit code 0
- **THEN** the `pkg/` directory contains JavaScript glue and a `.wasm` binary generated from `crates/game_core`

### Requirement: getProgramName API

The WASM module SHALL export a function named `getProgramName` that returns the string `Hello, My Dear World` when invoked from JavaScript after WASM initialization.

#### Scenario: getProgramName returns greeting

- **WHEN** the WASM module is initialized and `getProgramName()` is called from JavaScript
- **THEN** the return value is the string `Hello, My Dear World`

### Requirement: Workspace dependencies

The root `Cargo.toml` SHALL declare `wasm-bindgen` under `[workspace.dependencies]`, and `crates/game_core` SHALL reference it with `{ workspace = true }`.

#### Scenario: Shared dependency declaration

- **WHEN** a developer reads `crates/game_core/Cargo.toml`
- **THEN** `wasm-bindgen` is declared as a workspace dependency, not with a standalone version pin in the member crate alone

### Requirement: Dev watch for Rust sources

The project SHALL provide a development script that watches `crates/game_core` and rebuilds WASM on change using `cargo watch` and `wasm-pack`.

#### Scenario: Watch script documented

- **WHEN** a developer reads `package.json` scripts
- **THEN** a script exists (e.g. `dev:wasm`) that runs `cargo watch` to invoke `wasm-pack build` for `crates/game_core` on source changes

### Requirement: getCoreBuildInfo API

The WASM module SHALL export a function named `getCoreBuildInfo` that returns a JSON string with fields `layer` (`"core"`) and `version` (compact UTC datetime `YYYYMMDD.HHMMSS` of the last commit touching `crates/game_core/`, or `"unknown"`). Values SHALL be determined at WASM compile time via `build.rs` and embedded with `env!`.

#### Scenario: getCoreBuildInfo matches git

- **WHEN** the project is a git repository with at least one commit touching `crates/game_core/`
- **AND** WASM is rebuilt after the latest such commit
- **THEN** parsing `getCoreBuildInfo()` yields `version` equal to `TZ=UTC git log -1 --format=%cd --date=format:%Y%m%d.%H%M%S -- crates/game_core/`

#### Scenario: getCoreBuildInfo fallback without git

- **WHEN** git is unavailable during WASM build or no commit history exists for `crates/game_core/`
- **THEN** `getCoreBuildInfo()` returns JSON with `version: "unknown"`

### Requirement: ECS game world

The `crates/game_core` WASM module SHALL maintain a `bevy_ecs::World` containing unit entities. Each unit entity SHALL have a `Position` component with `x` and `y` fields as `f32` values in the range `[0, FIELD_WIDTH)` and `[0, FIELD_HEIGHT)` respectively, where `FIELD_WIDTH` is 25 (in tile units) and `FIELD_HEIGHT` is 19 (in tile units). Each spawned unit's position SHALL lie on a walkable tile of the world's `TileMapResource`.

#### Scenario: World contains spawned units

- **WHEN** `createGameWorld(unitCount, seed)` is called with `unitCount` greater than 0
- **THEN** the internal ECS world contains exactly `unitCount` entities with a `Position` component
- **THEN** each position has `0 <= x < 25` and `0 <= y < 19`

#### Scenario: Spawned units are on walkable tiles

- **WHEN** `createGameWorld(unitCount, seed)` is called with `unitCount` greater than 0
- **THEN** for each unit position, converting `(x, y)` to tile coordinates via `TileMapResource::world_to_tile` yields a tile where `is_walkable` is `true`

### Requirement: createGameWorld API

The WASM module SHALL export a function named `createGameWorld` that accepts `unitCount` (`u32`) and `seed` (`u64`), spawns `unitCount` units with pseudo-random positions derived from `seed` on walkable tiles of the generated tile map, and returns a handle to the game world.

#### Scenario: Deterministic spawn from seed

- **WHEN** `createGameWorld(3, 42)` is called twice in separate WASM sessions
- **THEN** both calls produce worlds whose `getUnitPositions()` JSON arrays are identical

#### Scenario: Unit count matches request

- **WHEN** `createGameWorld(3, seed)` is called
- **THEN** `getUnitPositions()` returns a JSON array of length 3

#### Scenario: Units spawn on walkable tiles only

- **WHEN** `createGameWorld(unitCount, seed)` is called
- **THEN** no unit's initial position maps to a blocked tile in `getTileMap()`

### Requirement: getUnitPositions API

The WASM module SHALL export a method `getUnitPositions` on the game world handle that returns a JSON string: an array of objects `{ "id": number, "x": number, "y": number }`, where `id` is the spawn index from `0` to `unitCount - 1`.

#### Scenario: JSON shape and coordinates

- **WHEN** a game world is created and `getUnitPositions()` is called
- **THEN** parsing the result yields an array of objects each with numeric `id`, `x`, and `y`
- **THEN** all `x` and `y` values are within the field bounds (0..25, 0..19)

#### Scenario: Positions are on walkable tiles

- **WHEN** a game world is created and `getUnitPositions()` is called
- **THEN** each unit's `(x, y)` maps to a walkable tile according to the world's `TileMapResource`

### Requirement: ECS workspace dependencies

The root `Cargo.toml` SHALL declare `bevy_ecs` and `rand` under `[workspace.dependencies]`, and `crates/game_core` SHALL reference them with `{ workspace = true }`.

#### Scenario: Shared dependency declaration

- **WHEN** a developer reads `crates/game_core/Cargo.toml`
- **THEN** `bevy_ecs` and `rand` are declared as workspace dependencies, not with standalone version pins in the member crate alone

### Requirement: Target and Speed components

Each unit entity in the ECS world SHALL have a `Target` component with `x` and `y` fields (`f32`, within field bounds) and a `Speed` component (`f32`, in tile-units per second).

#### Scenario: Spawned units have target and speed

- **WHEN** `createGameWorld(unitCount, seed)` is called with `unitCount` greater than 0
- **THEN** each spawned unit entity has `Position`, `Target`, and `Speed` components
- **THEN** each `Speed` value is 3.75 tile-units/second

### Requirement: Movement system

The ECS world SHALL include a movement system that updates each unit's `Position` toward the first waypoint in its `Path` at the rate defined by `Speed`, given a `delta_time` in seconds. Upon arrival (within 0.125 tile-units of a waypoint), the movement system SHALL remove that waypoint and send a `TargetReached` message when the path is complete.

#### Scenario: Position moves toward target

- **WHEN** a unit's `Position` is not at its `Target` and `tick(deltaMs)` is called with `deltaMs` greater than 0
- **THEN** the unit's `Position` moves closer to its `Target`
- **THEN** the distance between `Position` and `Target` decreases

#### Scenario: Unit arrives at target

- **WHEN** a unit's `Position` is within 0.125 tile-units of its current waypoint during a `tick` call
- **THEN** the movement system sends a `TargetReached` message for that entity

### Requirement: Target reached message

The ECS world SHALL maintain a `Messages<TargetReached>` resource. When a unit's path becomes empty, the movement system SHALL send a `TargetReached { entity }` message. A separate system `find_path_action` SHALL read unread `TargetReached` messages in the same tick and assign only an A*-validated path to a reachable walkable target.

#### Scenario: Movement sends message on arrival

- **WHEN** a unit's path becomes empty during a `tick` call
- **THEN** the movement system sends a `TargetReached` message with the unit's entity
- **THEN** the movement system does not assign a direct route through the map

#### Scenario: Assign system sets new target

- **WHEN** a `TargetReached` message is sent during a `tick` call
- **THEN** `find_path_action` assigns a new A*-validated path to that entity within field bounds (0..25, 0..19), or leaves it empty if no reachable target is found
- **THEN** the new target is assigned in the same tick, before `Messages<TargetReached>::update()` is called

### Requirement: tick API

The WASM module SHALL export a method `tick` on the game world handle that accepts `deltaMs` (`f32`, milliseconds) and advances the simulation by one step, running the movement system.

#### Scenario: tick advances simulation

- **WHEN** a game world is created and `tick(16.0)` is called
- **THEN** unit positions returned by `getUnitPositions()` differ from positions before the tick (unless all units are already at their targets and new targets coincide — acceptable edge case)

#### Scenario: Deterministic tick sequence

- **WHEN** two game worlds are created with `createGameWorld(3, 42)` and receive the same sequence of `tick(deltaMs)` calls
- **THEN** `getUnitPositions()` returns identical JSON after each tick in both worlds

### Requirement: TileMapResource coordinates

`TileMapResource` SHALL not store or expose a `TILE_SIZE` constant. Coordinate conversion methods SHALL operate in tile units:
- `world_to_tile(x, y)`: SHALL return `(x.floor() as u32, y.floor() as u32)`
- `tile_to_world(col, row)`: SHALL return `(col as f32 + 0.5, row as f32 + 0.5)`

#### Scenario: world_to_tile converts tile-units to grid

- **WHEN** `world_to_tile(5.3, 3.7)` is called
- **THEN** it returns `(5, 3)`

#### Scenario: tile_to_world returns tile center

- **WHEN** `tile_to_world(5, 3)` is called
- **THEN** it returns `(5.5, 3.5)`
