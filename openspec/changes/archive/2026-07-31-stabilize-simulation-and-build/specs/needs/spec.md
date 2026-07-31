## MODIFIED Requirements

### Requirement: Hungry colonist seeks berry bush
When a colonist receives a `Hungry` event, the system SHALL find the nearest available `FoodSource(BerryBush, _)` relative to that colonist's current position and create an `Eat` plan for it.

#### Scenario: Hungry colonist plans to go to nearest berry bush
- **WHEN** a colonist triggers `Hungry` and at least one charged berry bush exists
- **THEN** the selected bush minimizes distance from that colonist's position among available bushes
- **THEN** a `NeedsPlan` with kind `Eat` targets that bush

#### Scenario: No berry bushes available
- **WHEN** a colonist triggers `Hungry` but no charged berry bushes exist
- **THEN** no `NeedsPlan` is created

### Requirement: Tired colonist seeks bed
When a colonist receives a `Tired` event, the system SHALL find the nearest completed bed relative to that colonist's current position and create a `Sleep` plan for it.

#### Scenario: Tired colonist plans to go to nearest bed
- **WHEN** a colonist triggers `Tired` and at least one bed exists
- **THEN** the selected bed minimizes distance from that colonist's position
- **THEN** a `NeedsPlan` with kind `Sleep` targets that bed

#### Scenario: No beds available
- **WHEN** a colonist triggers `Tired` but no bed exists
- **THEN** no `NeedsPlan` is created

### Requirement: Initial world has three berry bushes
When the game world is created, it SHALL contain exactly three charged berry bushes on three distinct walkable tiles.

#### Scenario: Three bushes spawn at game start
- **WHEN** `createGameWorld` completes
- **THEN** exactly three `FoodSource(BerryBush, 5)` objects exist
- **THEN** their tiles are distinct and walkable

