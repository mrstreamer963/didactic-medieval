## MODIFIED Requirements

### Requirement: WASM crate build
The project SHALL build the current `crates/game_core` crate into the root `pkg/` directory using the standard npm build scripts. The documented paths and build metadata SHALL refer to `game_core`, not the removed `crates/core` path.

#### Scenario: Build current crate
- **WHEN** the user runs the standard WASM build command
- **THEN** the command builds `crates/game_core`
- **THEN** `pkg/` contains loadable JavaScript glue and a WASM binary

#### Scenario: Documentation matches the crate
- **WHEN** a developer follows README or devlog build instructions
- **THEN** all referenced Rust crate paths resolve to `crates/game_core`

