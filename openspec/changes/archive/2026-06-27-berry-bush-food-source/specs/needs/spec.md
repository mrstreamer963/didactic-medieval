# Needs (Satiation & Energy) — Delta

## MODIFIED Requirements

### Requirement: Hungry colonist seeks berry bush
When a colonist receives a `Hungry` event, the system SHALL find the nearest `FoodSource(BerryBush, _)` on the map and create a `NeedsPlan` with kind `Eat` targeting that bush's position.

#### Scenario: Hungry colonist plans to go to berry bush
- **WHEN** a colonist triggers `Hungry`
- **THEN** system searches `MapObjects` for the nearest `FoodSource(BerryBush, _)` and assigns `NeedsPlan { Eat, bush_pos }`

#### Scenario: No berry bushes available
- **WHEN** a colonist triggers `Hungry` but no berry bushes exist on the map
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

### Requirement: Initial world has three berry bushes
When the game world is created, `create_game_world` SHALL spawn 3 `FoodSource(BerryBush, 5)` on random walkable tiles.

#### Scenario: Three bushes spawn at game start
- **WHEN** `create_game_world` executes
- **THEN** 3 `FoodSource(BerryBush, 5)` are placed on random walkable tiles via `random_walkable_tile`

## REMOVED Requirements

### Requirement: Hungry colonist seeks campfire
**Reason**: Campfire replaced by physical berry bush objects with finite charges
**Migration**: Hungry colonists now search for `FoodSource(BerryBush, _)` instead of `Campfire`. See MODIFIED "Hungry colonist seeks berry bush".

#### Scenario: Hungry colonist plans to go to campfire
**Reason**: Same as above
**Migration**: No longer applicable

#### Scenario: No campfires available
**Reason**: Same as above
**Migration**: No longer applicable

### Requirement: Colonist eats at campfire
**Reason**: Eating now happens at berry bush locations
**Migration**: See MODIFIED "Colonist eats at berry bush"

#### Scenario: Colonist moves toward campfire
**Reason**: Replaced by berry bush movement
**Migration**: No longer applicable

#### Scenario: Colonist eats at campfire
**Reason**: Replaced by berry bush eating mechanic
**Migration**: No longer applicable
