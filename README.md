
# Didactic Medieval

Игровой прототип на React/PixiJS и Rust/WASM. Симуляция хранится в crate
`crates/game_core`, а сгенерированные WASM-файлы помещаются в корневой
каталог `pkg/`.

## Запуск

```bash
npm install
rustup target add wasm32-unknown-unknown
cargo install wasm-pack cargo-watch
npm run dev
```

Production-сборка без обязательного `wasm-opt`:

```bash
npm run build
npm run preview
```

Обычная команда `build:wasm` использует `--dev --no-opt`, поэтому не скачивает
опциональный `wasm-opt` интерактивно. Если локально установлен `wasm-opt` и
нужна оптимизированная release-сборка, используйте `npm run build:wasm:optimized`.

## Модель координат и API

Карта имеет размер 25×19 клеток. В Rust/WASM координаты работают в
`tile-units`: клетка `(col, row)` занимает `[col, col + 1) × [row, row + 1)`,
а её центр — `(col + 0.5, row + 0.5)`. PixiJS только масштабирует эти координаты
в экранные 32×32 пикселя.

Компонент `Path` содержит `Vec<(f32, f32)>` мировых координат waypoint-ов,
которые всегда соответствуют центрам проходимых клеток и строятся через A*.

Основной фронтенд создаёт 3 стартовых юнита:

```text
createGameWorld(3, seed)
```

Публичные методы WASM: `createGameWorld(unitCount, seed)`, `tick(deltaMs)`,
`getUnitPositions()`, `getUnitStates()`, `getTileMap()`, `build(col, row, kind)`.
