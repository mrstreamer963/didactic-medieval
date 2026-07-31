# Needs (Satiation & Energy)

**Purpose**: Colonists have basic biological needs (hunger and fatigue) that decay over time and must be fulfilled by visiting appropriate buildings.

## Requirements

### Requirement: Colonist satiation decreases over time
Every colonist SHALL have a `Satiation` value (0–100) that decreases by 0.8/sec while the simulation runs.

#### Scenario: Satiation decreases each tick
- **WHEN** simulation runs for 1 second
- **THEN** each colonist's satiation decreases by approximately 0.8

### Requirement: Colonist energy decreases over time
Every colonist SHALL have an `Energy` value (0–100) that decreases by 0.4/sec while the simulation runs.

#### Scenario: Energy decreases each tick
- **WHEN** simulation runs for 1 second
- **THEN** each colonist's energy decreases by approximately 0.4

### Requirement: Colonist becomes hungry when satiation is low
When a colonist's `Satiation` drops below 30, the system SHALL emit a `Hungry` event and apply the `HungryDebuff` component.

#### Scenario: Hungry event on low satiation
- **WHEN** a colonist's satiation drops below 30 for the first time
- **THEN** system emits `Hungry` event and adds `HungryDebuff` component

#### Scenario: No duplicate Hungry events
- **WHEN** a colonist already has `HungryDebuff` and satiation is still below 30
- **THEN** no additional `Hungry` event is emitted

### Requirement: Colonist becomes full when satiation recovers
When a colonist's `Satiation` rises above 90 and they have `HungryDebuff`, the system SHALL emit a `Sated` event and remove `HungryDebuff`.

#### Scenario: Sated event on recovery
- **WHEN** a colonist's satiation rises above 90 while `HungryDebuff` is active
- **THEN** system emits `Sated` event and removes `HungryDebuff`

### Requirement: Colonist becomes tired when energy is low
When a colonist's `Energy` drops below 30, the system SHALL emit a `Tired` event and apply the `TiredDebuff` component.

#### Scenario: Tired event on low energy
- **WHEN** a colonist's energy drops below 30 for the first time
- **THEN** system emits `Tired` event and adds `TiredDebuff` component

### Requirement: Colonist becomes rested when energy recovers
When a colonist's `Energy` rises above 90 and they have `TiredDebuff`, the system SHALL emit a `Rested` event and remove `TiredDebuff`.

#### Scenario: Rested event on recovery
- **WHEN** a colonist's energy rises above 90 while `TiredDebuff` is active
- **THEN** system emits `Rested` event and removes `TiredDebuff`

### Requirement: Hungry colonist seeks berry bush
When a colonist receives a `Hungry` event, the system SHALL find the nearest `FoodSource(BerryBush, _)` on the map and create a `NeedsPlan` with kind `Eat` targeting that bush's position.

#### Scenario: Hungry colonist plans to go to berry bush
- **WHEN** a colonist triggers `Hungry`
- **THEN** system searches `MapObjects` for the nearest `FoodSource(BerryBush, _)` and assigns `NeedsPlan { Eat, bush_pos }`

#### Scenario: No berry bushes available
- **WHEN** a colonist triggers `Hungry` but no berry bushes exist on the map
- **THEN** no `NeedsPlan` is created; colonist continues wandering

### Requirement: Tired colonist seeks bed
When a colonist receives a `Tired` event, the system SHALL find the nearest bed on the map and create a `NeedsPlan` with kind `Sleep` targeting that bed's position.

#### Scenario: Tired colonist plans to go to bed
- **WHEN** a colonist triggers `Tired`
- **THEN** system searches `MapObjects` for the nearest `Bed` and assigns `NeedsPlan { Sleep, bed_pos }`

#### Scenario: No beds available
- **WHEN** a colonist triggers `Tired` but no beds exist on the map
- **THEN** no `NeedsPlan` is created; colonist continues wandering

### Requirement: Colonist executes needs plan
When a colonist has a `NeedsPlan`, the system SHALL move the colonist toward the plan target. Upon arrival, the colonist SHALL restore satiation (eat) or energy (sleep) at the appropriate rate.

#### Scenario: Colonist moves toward berry bush
- **WHEN** a colonist has `NeedsPlan { Eat, target }` and is farther than one tile from target
- **THEN** system calculates an A* path to the target and moves the colonist along it

#### Scenario: Colonist eats at berry bush
- **WHEN** a colonist with `NeedsPlan { Eat, target }` arrives at the target
- **THEN** satiation increases by ~15/sec until above 95

#### Scenario: Sated decrements berry bush charge
- **WHEN** a colonist with `NeedsPlan { Eat, target }` becomes Sated and target is a berry bush
- **THEN** the bush's charge is decremented by 1; if charges reach 0, the bush is removed from the map

#### Scenario: Colonist sleeps at bed
- **WHEN** a colonist with `NeedsPlan { Sleep, target }` arrives at the target
- **THEN** energy increases by ~7/sec until above 95

#### Scenario: Needs plan has priority over wandering
- **WHEN** a colonist has an active `NeedsPlan`
- **THEN** `find_path_action` does NOT assign a random wandering target

### Requirement: Initial world has three berry bushes
When the game world is created, `create_game_world` SHALL spawn 3 `FoodSource(BerryBush, 5)` on random walkable tiles.

#### Scenario: Three bushes spawn at game start
- **WHEN** `create_game_world` executes
- **THEN** 3 `FoodSource(BerryBush, 5)` are placed on random walkable tiles via `random_walkable_tile`

### Requirement: Sated/Rested clears needs plan
When a colonist receives a `Sated` or `Rested` event, the system SHALL remove the matching `NeedsPlan`.

#### Scenario: Sated clears eat plan
- **WHEN** a colonist with `NeedsPlan { Eat, ... }` receives `Sated`
- **THEN** the `NeedsPlan` is removed; colonist returns to wandering

#### Scenario: Rested clears sleep plan
- **WHEN** a colonist with `NeedsPlan { Sleep, ... }` receives `Rested`
- **THEN** the `NeedsPlan` is removed; colonist returns to wandering

### Requirement: Frontend displays need status icons
The frontend SHALL display status icons (hungry/tired) above colonists who have active debuffs, updating every frame.

#### Scenario: Hungry icon appears
- **WHEN** a colonist has `hungry: true` in `getUnitStates()`
- **THEN** a yellow hungry icon is rendered above the colonist's sprite

#### Scenario: Tired icon appears
- **WHEN** a colonist has `tired: true` in `getUnitStates()`
- **THEN** a blue tired icon is rendered above the colonist's sprite

#### Scenario: Both icons appear
- **WHEN** a colonist has both `hungry: true` and `tired: true`
- **THEN** both icons are rendered side by side above the colonist
