# Food Source Charges

**Purpose**: Some map objects (berry bushes) are finite food sources with a limited number of charges that get consumed when colonists eat.

## Requirements

### Requirement: FoodSource has finite charges
Every `MapTileObject::FoodSource` SHALL have a `u8` charges value. When a colonist completes a full eat cycle (becomes Sated), one charge SHALL be decremented.

#### Scenario: Charge decremented on sated
- **WHEN** a colonist with `NeedsPlan { Eat, target }` targeting a `FoodSource(BerryBush, _)` becomes Sated
- **THEN** the system decrements the bush's charge by 1

#### Scenario: Zero charges removes bush
- **WHEN** a bush's charges reach 0 after decrement
- **THEN** the bush is removed from `MapObjects` (tile becomes `None`)

#### Scenario: Colonist cannot eat from empty bush
- **WHEN** a colonist searches for food but all bushes have 0 charges (or are gone)
- **THEN** no `NeedsPlan` is created; colonist continues wandering

### Requirement: Initial berry bushes have 5 charges
When the game world is created, each spawned berry bush SHALL start with exactly 5 charges.

#### Scenario: Starting bushes have 5 charges
- **WHEN** `create_game_world` spawns a berry bush
- **THEN** its charges equal 5

### Requirement: Constructed berry bushes have 5 charges
When a berry bush construction completes, the resulting bush SHALL have 5 charges.

#### Scenario: Built bush has 5 charges
- **WHEN** a `ConstructionSite(BerryBush)` completes
- **THEN** the resulting `FoodSource(BerryBush, 5)` is placed in `MapObjects`
