## 1. Rust ECS — Типы и ресурсы

- [x] 1.1 Добавить `BerryBush` в `ObjectKind` в `resources.rs`, удалить `Campfire`
- [x] 1.2 Добавить `FoodSource(ObjectKind, u8)` в `MapTileObject` в `resources.rs`

## 2. Rust ECS — Системы

- [x] 2.1 В `systems.rs`: обновить `find_nearest_object` — искать и `Building(k)`, и `FoodSource(k, _)`
- [x] 2.2 В `needs_decision`: искать `FoodSource(BerryBush, _)` вместо `Campfire` при голоде
- [x] 2.3 В `needs_decision`: при `Sated` читать `NeedsPlan` цели, находить тайл куста, декрементить `charges`; если `charges == 0` → `map_objects.tiles[idx] = None`
- [x] 2.4 В `construction_system`: добавить `ObjectKind::BerryBush` с `max_progress = 40.0`, не проверять blocked для BerryBush
- [x] 2.5 Удалить `ObjectKind::Campfire` из всех match-выражений в systems.rs

## 3. Rust ECS — Мир и интеграция

- [x] 3.1 В `world.rs`: в `build()` заменить `"campfire"` на `"berrybush"`
- [x] 3.2 В `get_map_objects()`, `get_construction_progress()`, `get_unit_states()`: заменить `Campfire` на `BerryBush` в match, добавить `FoodSource` вариант с kind "berrybush" и charges в JSON
- [x] 3.3 В `create_game_world`: после спавна юнитов заспавнить 3 `FoodSource(BerryBush, 5)` на случайных тайлах

## 4. Фронтенд — Рендеринг

- [x] 4.1 В `BuildingRenderer.ts`: добавить `'berrybush'` в тип `MapObject.kind` и `'berrybush'` в тип `underlying`; убрать `'campfire'`
- [x] 4.2 В `drawBuildingShape`: добавить `case 'berrybush'` — зелёный полукруглый куст `0x2d8a4e` с 3-4 фиолетовыми кругами-ягодами `0x7b2d8e`
- [x] 4.3 Удалить `case 'campfire'` из `drawBuildingShape`

## 5. Фронтенд — UI

- [x] 5.1 В `BuildToolbar.tsx`: кнопка "Куст" вместо "Костёр", значение `'berrybush'`
- [x] 5.2 В `UnitsCanvas.tsx`: `BuildMode` — `'berrybush'` вместо `'campfire'`

## 6. Тесты

- [x] 6.1 Обновить `BuildingRenderer.test.ts`: заменить `'campfire'` на `'berrybush'` и `'berrybush'` в тестовых данных
- [x] 6.2 Обновить Rust-тесты в `systems.rs`: заменить `Campfire` на `BerryBush`, проверить логику зарядов

## 7. Сборка и проверка

- [x] 7.1 Собрать WASM (`wasm-pack build`) и проверить, что нет ошибок компиляции
- [x] 7.2 Запустить фронтенд-тесты (`npm test` или `vitest`)
- [x] 7.3 Визуально проверить: кусты отображаются на карте, еда работает, заряды тратятся
