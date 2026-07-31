# Building — Delta

## MODIFIED Requirements

### Requirement: Player can enter build mode
The frontend SHALL provide a toolbar with buttons for three building types: Wall, Bed, Berry Bush.

#### Scenario: Select build mode
- **WHEN** player clicks the "Стена" button
- **THEN** build mode is activated for walls; the button is highlighted

#### Scenario: Select berry bush build
- **WHEN** player clicks the "Куст" button
- **THEN** build mode is activated for berry bushes; the button is highlighted

#### Scenario: Deselect build mode
- **WHEN** player clicks the active build mode button or presses Escape
- **THEN** build mode is deactivated

#### Scenario: Build mode cycling
- **WHEN** build mode is active and player clicks a different building button
- **THEN** build mode switches to the new building type

### Requirement: Buildings are rendered on the map
The frontend SHALL render all placed buildings as visual sprites on a dedicated buildings layer.

#### Scenario: Wall is rendered as gray rectangle
- **WHEN** a wall is placed on the map
- **THEN** a gray 32×32 rectangle appears at that tile position

#### Scenario: Bed is rendered as brown shape
- **WHEN** a bed is placed on the map
- **THEN** a brown procedural sprite appears at that tile position

#### Scenario: Berry bush is rendered as green bush with berries
- **WHEN** a berry bush is placed on the map
- **THEN** a green semi-circular bush (`0x2d8a4e`) with 3-4 purple berry circles (`0x7b2d8e`) appears at that tile position

#### Scenario: Building sprites persist across ticks
- **WHEN** the simulation ticks
- **THEN** building sprites remain in sync with `getMapObjects()` data

### Requirement: WASM API exposes building methods
The WASM bridge SHALL expose `build(col, row, kind)`, `getMapObjects()`, and `getUnitStates()` methods.

#### Scenario: Build method creates request
- **WHEN** `world.build(5, 10, "berrybush")` is called from JS
- **THEN** ECS receives a `BuildRequest` for a berry bush at col=5, row=10

#### Scenario: Get map objects returns JSON with berry bush
- **WHEN** `world.getMapObjects()` is called
- **THEN** a JSON array of all placed objects is returned, including `{kind: "berrybush"}` entries

#### Scenario: Get unit states returns JSON
- **WHEN** `world.getUnitStates()` is called
- **THEN** a JSON array with each colonist's satiation, energy, and debuff status is returned

## REMOVED Requirements

### Requirement: Campfire is rendered with animation
**Reason**: Campfire replaced by berry bush
**Migration**: See MODIFIED "Berry bush is rendered as green bush with berries"

#### Scenario: Campfire visual
**Reason**: Replaced by berry bush rendering
**Migration**: No longer applicable
