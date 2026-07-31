## MODIFIED Requirements

### Requirement: Production build
The project SHALL produce a static production build via the standard npm build command in a supported development environment.

#### Scenario: Build succeeds without optional optimizer
- **WHEN** the user runs `npm run build` on an environment with Rust, wasm-pack, and the WASM target but without an optional `wasm-opt` binary
- **THEN** the WASM package is generated successfully without requiring an interactive download
- **THEN** TypeScript compilation and Vite bundling complete with exit code 0
- **THEN** optimized WASM remains available through an explicitly documented optional toolchain

