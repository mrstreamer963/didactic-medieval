## MODIFIED Requirements

### Requirement: Simulation delta is multiplied by speed
When `gameSpeed > 0`, the simulation SHALL receive `ticker.deltaMS * gameSpeed`. When `gameSpeed === 0`, the simulation SHALL NOT be advanced and rendering SHALL continue from the last state.

#### Scenario: Speed affects simulation tick rate
- **GIVEN** speed is `5`
- **WHEN** the renderer advances by 16ms
- **THEN** the simulation receives approximately 80ms

#### Scenario: Pause freezes simulation
- **GIVEN** speed is `0`
- **WHEN** the renderer frame callback runs
- **THEN** no simulation step is called
- **THEN** unit positions, needs, construction progress, jobs, and targets remain unchanged
- **THEN** the current scene remains visible

