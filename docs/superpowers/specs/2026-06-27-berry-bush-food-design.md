# Berry Bush — еда с ресурсом

## Цель
Заменить абстрактное питание у костра на физический объект "куст с ягодами" на карте, у которого есть конечный ресурс (заряды). Когда заряды кончаются — куст исчезает.

## Изменения в Rust (ECS)

### resources.rs
- Удалить `ObjectKind::Campfire`
- Добавить `ObjectKind::BerryBush`
- Добавить `MapTileObject::FoodSource(ObjectKind, u8)` — `u8` = количество зарядов (5)
- Начальные кусты (3 шт.) спавнятся в `create_game_world` через `random_walkable_tile`

### systems.rs
- `find_nearest_object` — ищет и `Building(k)`, и `FoodSource(k, _)`
- `needs_decision` — при `Hungry` ищет `FoodSource(BerryBush, _)` вместо `Campfire`
- `needs_decision` — при `Sated`: читает `NeedsPlan` цели, находит тайл, декрементит `charges`; если `charges == 0` → `map_objects.tiles[idx] = None`
- `construction_system` — добавить `ObjectKind::BerryBush` (max_progress = 40.0, не блокирует проход)
- Удалить `Campfire` из всех match-выражений

### world.rs
- Убрать `"campfire"` из `build()`, добавить `"berrybush"`
- Убрать `ObjectKind::Campfire` из `get_unit_states` и `get_map_objects`
- В `create_game_world`: после спавна юнитов, заспавнить 3 `FoodSource(BerryBush, 5)` на случайных тайлах

## Изменения на фронтенде (TypeScript/Pixi.js)

### BuildingRenderer.ts
- Добавить `case 'berrybush'` в `drawBuildingShape`
  - Зелёный полукруглый куст (ellipse или rect со скруглениями) — `0x2d8a4e`
  - 3-4 фиолетовых круга-ягодки — `0x7b2d8e`
- Тип `MapObject.kind`: добавить `'berrybush'`, убрать `'campfire'`

### BuildToolbar.tsx
- Кнопка "Куст" вместо "Костёр", значение `'berrybush'`

### UnitsCanvas.tsx
- `BuildMode`: `'wall' | 'bed' | 'berrybush' | null`

## Потребление
- Единица ест 15 ед. насыщения/сек (EAT_RATE)
- 1 заряд тратится за полный цикл голод→сыт
- bush исчезает при charges=0

## Строительство
- max_progress = 40.0 (чуть быстрее костра)
- Не блокирует тайл (walkable остаётся true)
