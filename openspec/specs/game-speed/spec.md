# Game Speed Controls

## Purpose

Add three simulation speeds (x1, x5, x10) and a pause mode, controllable via keyboard shortcuts and UI buttons.

## Requirements

### Requirement: Simulation speed is a state at the App level
The app SHALL hold a `gameSpeed` state with valid values `0` (pause), `1`, `5`, `10`. Default SHALL be `1`.

#### Scenario: Default speed is 1
- **WHEN** the app initialises
- **THEN** `gameSpeed` is `1`

### Requirement: Keyboard shortcuts toggle speed
The app SHALL handle the following keyboard events:
- `Space` — toggle between pause (0) and the previous non-zero speed (or 1 if none)
- `1` — set speed to `1`
- `2` — set speed to `5`
- `3` — set speed to `10`

#### Scenario: Space toggles pause
- **GIVEN** speed is `1`
- **WHEN** user presses Space
- **THEN** speed changes to `0`
- **WHEN** user presses Space again
- **THEN** speed returns to `1`

#### Scenario: Number keys set speed directly
- **WHEN** user presses `2`
- **THEN** speed changes to `5`
- **WHEN** user presses `3`
- **THEN** speed changes to `10`
- **WHEN** user presses `1`
- **THEN** speed changes to `1`

### Requirement: UI buttons display and control speed
A row of speed control buttons SHALL be displayed below the build toolbar. The button matching the current speed SHALL be visually highlighted. Buttons SHALL be labelled in Russian: ⏸ Пауза, x1, x5, x10.

#### Scenario: Buttons reflect current speed
- **GIVEN** speed is `5`
- **WHEN** UI renders
- **THEN** the "x5" button has an `active` CSS class

#### Scenario: Clicking a button changes speed
- **WHEN** user clicks "Пауза"
- **THEN** speed changes to `0`
- **WHEN** user clicks "x10"
- **THEN** speed changes to `10`

### Requirement: Simulation delta is multiplied by speed
When `gameSpeed > 0`, the `world.tick()` call SHALL receive `ticker.deltaMS * gameSpeed`. When `gameSpeed === 0`, `world.tick()` SHALL NOT be called; rendering continues so the scene remains visible.

#### Scenario: Speed affects simulation tick rate
- **GIVEN** speed is `5`
- **WHEN** `ticker.deltaMS` is 16ms
- **THEN** `world.tick(80)` is called

#### Scenario: Pause freezes simulation
- **GIVEN** speed is `0`
- **WHEN** ticker fires
- **THEN** `world.tick()` is NOT called
- **THEN** unit positions and states remain unchanged

### Requirement: MAX_DELTA_MS is increased to support x10
The Rust-side `MAX_DELTA_MS` constant SHALL be increased from 100 to 200 so that x10 speed at 60fps (167ms) is not capped.

#### Scenario: x10 at 60fps passes without cap
- **GIVEN** `ticker.deltaMS` is ~16ms
- **WHEN** speed is 10
- **THEN** passed delta is ~167ms, which is less than 200ms

### Requirement: Keyboard events do not conflict with existing handlers
The Escape key handler (which clears build mode) SHALL continue to work. Space SHALL NOT trigger page scroll.

#### Scenario: Escape still clears build mode
- **WHEN** user presses Escape
- **THEN** build mode is cleared (unchanged behavior)

#### Scenario: Space does not scroll page
- **WHEN** user presses Space
- **THEN** `e.preventDefault()` is called on the event

## Implementation Notes

### Files to modify
- `src/App.tsx` — add `gameSpeed` state, pass to `UnitsCanvas`, add keyboard handler for speed keys
- `src/UnitsCanvas.tsx` — accept `gameSpeed` prop, multiply delta in ticker callback
- `src/SpeedControls.tsx` — new component for speed buttons
- `crates/game_core/src/systems.rs` — increase `MAX_DELTA_MS` from 100 to 200
- `src/App.css` — styles for speed controls

### Keyboard handler
The keyboard SHALL be handled via `window.addEventListener('keydown', ...)` in App.tsx, alongside the existing Escape handler. Space SHALL call `e.preventDefault()` to prevent scrolling.
