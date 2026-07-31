# Devlog

## Инструменты

```bash
npm install
rustup target add wasm32-unknown-unknown
cargo install wasm-pack cargo-watch
```

## Vite + React

```bash
npm run dev      # build:wasm без wasm-opt + watch Rust + Vite
npm run build    # стандартная production-сборка без обязательного wasm-opt
npm run preview  # просмотр dist после build
```

Для явно оптимизированной WASM-сборки при наличии локального `wasm-opt`:

```bash
npm run build:wasm:optimized
```

## Rust / WASM

```bash
npm run build:wasm   # crates/game_core → корневой pkg/, no-opt
npm run dev:wasm     # watch crates/game_core и повторная no-opt-сборка
```

Приветствие на странице берётся из `getProgramName()` в WASM-модуле
`crates/game_core`.

## ECS, юниты и координаты

`crates/game_core` использует `bevy_ecs` для хранения юнитов. Карта имеет
25×19 клеток, а все координаты Rust/WASM выражены в `tile-units`, не в
пикселях: `tile_to_world(col, row)` возвращает центр `(col + 0.5, row + 0.5)`.
Экранный рендерер умножает их на 32.

Компонент `Path` содержит мировые waypoint-ы `(f32, f32)` в центрах
проходимых клеток. Маршруты создаются A* и не используют заблокированные
клетки.

WASM API:

- `createGameWorld(unitCount, seed)` — создаёт заданное число юнитов; фронтенд
  использует 3 стартовых юнита.
- `getUnitPositions()` — JSON-массив `[{ "id", "x", "y" }, ...]` в tile-units.
- `tick(deltaMs)` — один шаг симуляции с ограничением delta до 200 мс.
- `getTileMap()` — карта 25×19 с обозначениями `G` (проходимая) и `B` (blocked).
- `build(col, row, kind)` — ставит заявку на стену, кровать или куст только на
  свободную проходимую клетку.

Кнопка «Перегенерировать» создаёт новый мир с другим seed.

## Build version

Номер версии каждого слоя — компактная дата-время UTC (`YYYYMMDD.HHMMSS`)
последнего git-коммита, затронувшего этот слой:

- **frontend** — последний коммит по `src/`
- **core** — последний коммит по `crates/game_core/`

В футере приложения и через API доступны `getFrontendBuildInfo()`,
`getCoreBuildInfo()` и `getBuildInfo()`.

Сверка с git:

```bash
TZ=UTC git log -1 --format=%cd --date=format:%Y%m%d.%H%M%S -- src/
TZ=UTC git log -1 --format=%cd --date=format:%Y%m%d.%H%M%S -- crates/game_core/
```

Если git недоступен — версия `unknown`.
