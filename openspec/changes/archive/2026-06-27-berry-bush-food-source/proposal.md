## Why

Абстрактное питание у костра не создаёт интересных игровых решений. Физический объект "куст с ягодами" с конечным ресурсом добавляет depth: нужно управлять ресурсами, искать новые кусты, строить их. Механика конечных зарядов создаёт естественный цикл потребления и восполнения.

## What Changes

- **BREAKING**: `ObjectKind::Campfire` удаляется, заменяется на `ObjectKind::BerryBush`
- Добавляется `MapTileObject::FoodSource(ObjectKind, u8)` — объект с конечными зарядами
- 3 куста по 5 зарядов спавнятся на старте на случайных тайлах
- Голодный колонист ищет `FoodSource(BerryBush, _)` вместо `Campfire`
- При насыщении: тратится 1 заряд куста; при charges=0 куст исчезает
- В тулбаре "Костёр" заменяется на "Куст"
- Рендеринг куста: зелёный полукруг с фиолетовыми ягодами
- max_progress строительства = 40.0, куст не блокирует проход

## Capabilities

### New Capabilities
- `food-source-charges`: механика конечных зарядов на объекте-источнике еды (расход заряда при насыщении, уничтожение при charges=0)

### Modified Capabilities
- `needs`: требование "Hungry colonist seeks campfire" меняется на "Hungry colonist seeks berry bush"; добавляется расход заряда при насыщении; начальные кусты спавнятся в мире вместо абстрактного костра
- `building`: тип "Campfire" заменяется на "BerryBush" в тулбаре и рендеринге; начальное состояние мира содержит 3 куста
- `construction-queue`: max_progress для Campfire (60) заменяется на BerryBush (40); куст не блокирует проход

## Impact

- Rust ECS: `resources.rs`, `systems.rs`, `world.rs` — изменения типов, логики поиска и потребления
- TypeScript/Pixi.js: `BuildingRenderer.ts` — новый рендеринг куста; `BuildToolbar.tsx` — кнопка "Куст"; `UnitsCanvas.tsx` — тип билд-режима
- Спеки: needs, building, construction-queue — изменение требований; новая спека food-source-charges
