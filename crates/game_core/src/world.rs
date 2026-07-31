use bevy_ecs::prelude::*;
use rand::SeedableRng;
use rand::rngs::StdRng;
use wasm_bindgen::prelude::*;

use crate::components::{
    AssignedJob, Energy, HungryDebuff, NeedKind, NeedsPlan, Path, Position, Satiation, Speed,
    TiredDebuff, UnitId,
};
use crate::events::{BuildRequest, Hungry, Rested, Sated, TargetReached, Tired};
use crate::pathfinding::{astar, random_reachable_path, tile_path_to_waypoints};
use crate::resources::{
    ConstructionQueue, DeltaTime, MapObjects, MapTileObject, ObjectKind, SimulationRng,
    TileMapResource,
};
use crate::systems::{
    DEFAULT_SPEED, MAX_DELTA_MS, construction_progress_system, construction_system,
    execute_needs_plan, find_path_action, job_assignment_system, move_along_path, needs_accrual,
    needs_decision, needs_event_check, release_job_on_needs,
};

pub const FIELD_WIDTH: f32 = 25.0;
pub const FIELD_HEIGHT: f32 = 19.0;

fn initial_path(
    start: (u32, u32),
    tile_map: &TileMapResource,
    rng: &mut StdRng,
) -> Vec<(f32, f32)> {
    let tile_path = random_reachable_path(start, tile_map, rng)
        .or_else(|| astar(start, start, tile_map))
        .expect("a spawned unit must start on a walkable tile");
    tile_path_to_waypoints(&tile_path, tile_map)
}

#[wasm_bindgen]
pub struct GameWorld {
    world: World,
    schedule: Schedule,
}

#[wasm_bindgen(js_name = createGameWorld)]
pub fn create_game_world(unit_count: u32, seed: u64) -> GameWorld {
    let mut world = World::new();
    let mut rng = StdRng::seed_from_u64(seed);

    let tile_map = TileMapResource::new(seed);
    let mut map_objects = MapObjects::new(tile_map.cols, tile_map.rows);

    world.init_resource::<Messages<TargetReached>>();
    world.init_resource::<Messages<Hungry>>();
    world.init_resource::<Messages<Tired>>();
    world.init_resource::<Messages<Sated>>();
    world.init_resource::<Messages<Rested>>();
    world.init_resource::<Messages<BuildRequest>>();

    world.insert_resource(ConstructionQueue::new());

    for id in 0..unit_count {
        let (col, row) = tile_map.random_walkable_tile(&mut rng);
        let (x, y) = tile_map.tile_to_world(col, row);
        world.spawn((
            UnitId(id),
            Position { x, y },
            Path {
                waypoints: initial_path((col, row), &tile_map, &mut rng),
            },
            Speed(DEFAULT_SPEED),
            Satiation(100.0),
            Energy(100.0),
        ));
    }

    for (col, row) in tile_map
        .shuffled_walkable_tiles(&mut rng)
        .into_iter()
        .take(3)
    {
        let idx = (row * tile_map.cols + col) as usize;
        if idx < map_objects.tiles.len() {
            map_objects.tiles[idx] = Some(MapTileObject::FoodSource(ObjectKind::BerryBush, 5));
        }
    }

    world.insert_resource(tile_map);
    world.insert_resource(map_objects);
    world.insert_resource(SimulationRng(rng));

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            needs_accrual,
            needs_event_check,
            needs_decision,
            execute_needs_plan,
            release_job_on_needs,
            (find_path_action, construction_system),
            job_assignment_system,
            construction_progress_system,
            move_along_path,
        )
            .chain(),
    );

    GameWorld { world, schedule }
}

#[wasm_bindgen]
impl GameWorld {
    #[wasm_bindgen(js_name = tick)]
    pub fn tick(&mut self, delta_ms: f32) {
        let capped = delta_ms.min(MAX_DELTA_MS);
        let delta_secs = capped / 1000.0;
        self.world.insert_resource(DeltaTime(delta_secs));
        self.schedule.run(&mut self.world);
        self.world
            .resource_mut::<Messages<TargetReached>>()
            .update();
        self.world.resource_mut::<Messages<Hungry>>().update();
        self.world.resource_mut::<Messages<Tired>>().update();
        self.world.resource_mut::<Messages<Sated>>().update();
        self.world.resource_mut::<Messages<Rested>>().update();
        self.world.resource_mut::<Messages<BuildRequest>>().update();
    }

    #[wasm_bindgen(js_name = getTileMap)]
    pub fn get_tile_map(&self) -> String {
        let map = self.world.resource::<TileMapResource>();
        let mut json = String::from(r#"{"cols":25,"rows":19,"tiles":["#);
        for row in 0..map.rows {
            if row > 0 {
                json.push(',');
            }
            json.push('"');
            for col in 0..map.cols {
                let idx = (row * map.cols + col) as usize;
                json.push(if map.tiles[idx] { 'G' } else { 'B' });
            }
            json.push('"');
        }
        json.push_str("]}");
        json
    }

    #[wasm_bindgen(js_name = getUnitStates)]
    pub fn get_unit_states(&mut self) -> String {
        let assigned_map: Vec<(Entity, u32, u32, ObjectKind)> = {
            let queue = self.world.resource::<ConstructionQueue>();
            queue
                .jobs
                .iter()
                .flat_map(|job| {
                    let kind = job.kind;
                    job.assigned_units
                        .iter()
                        .map(move |&e| (e, job.col, job.row, kind))
                })
                .collect()
        };
        let mut json = String::from("[");
        let mut first = true;
        let mut query = self.world.query::<(
            Entity,
            &UnitId,
            &Satiation,
            &Energy,
            Option<&HungryDebuff>,
            Option<&TiredDebuff>,
            Option<&NeedsPlan>,
            &Speed,
            Option<&AssignedJob>,
        )>();
        for (entity, id, sat, ene, hungry, tired, plan, speed, assigned) in query.iter(&self.world)
        {
            if !first {
                json.push(',');
            }
            first = false;
            use std::fmt::Write as _;
            let plan_str = match plan {
                Some(p) if p.kind == NeedKind::Eat => "eat",
                Some(p) if p.kind == NeedKind::Sleep => "sleep",
                _ => "",
            };
            let assigned_job = if assigned.is_some() {
                let mut job_json = String::from("null");
                for &(e, col, row, kind) in &assigned_map {
                    if e == entity {
                        let kind_str = match kind {
                            ObjectKind::Wall => "wall",
                            ObjectKind::Bed => "bed",
                            ObjectKind::BerryBush => "berrybush",
                        };
                        job_json = String::new();
                        let _ = write!(
                            job_json,
                            r#"{{"col":{},"row":{},"kind":"{}"}}"#,
                            col, row, kind_str
                        );
                        break;
                    }
                }
                job_json
            } else {
                String::from("null")
            };
            let _ = write!(
                json,
                r#"{{"id":{},"satiation":{},"energy":{},"hungry":{},"tired":{},"needsPlan":"{}","speed":{},"assignedJob":{}}}"#,
                id.0,
                sat.0,
                ene.0,
                hungry.is_some(),
                tired.is_some(),
                plan_str,
                speed.0,
                assigned_job
            );
        }
        json.push(']');
        json
    }

    #[wasm_bindgen(js_name = build)]
    pub fn build(&mut self, col: u32, row: u32, kind: &str) {
        let object_kind = match kind {
            "wall" => ObjectKind::Wall,
            "bed" => ObjectKind::Bed,
            "berrybush" => ObjectKind::BerryBush,
            _ => return,
        };
        self.world
            .resource_mut::<Messages<BuildRequest>>()
            .write(BuildRequest {
                col,
                row,
                kind: object_kind,
            });
    }

    #[wasm_bindgen(js_name = getMapObjects)]
    pub fn get_map_objects(&self) -> String {
        let map = self.world.resource::<MapObjects>();
        let tile_map = self.world.resource::<TileMapResource>();
        let mut json = String::from("[");
        let mut first = true;
        for row in 0..tile_map.rows {
            for col in 0..tile_map.cols {
                let idx = (row * tile_map.cols + col) as usize;
                if idx >= map.tiles.len() {
                    continue;
                }
                if let Some(obj) = &map.tiles[idx] {
                    if !first {
                        json.push(',');
                    }
                    first = false;
                    match obj {
                        MapTileObject::Building(kind) => {
                            let kind_str = match kind {
                                ObjectKind::Wall => "wall",
                                ObjectKind::Bed => "bed",
                                ObjectKind::BerryBush => "berrybush",
                            };
                            use std::fmt::Write as _;
                            let _ =
                                write!(json, r#"{{"col":{col},"row":{row},"kind":"{kind_str}"}}"#);
                        }
                        MapTileObject::FoodSource(kind, charges) => {
                            let kind_str = match kind {
                                ObjectKind::Wall => "wall",
                                ObjectKind::Bed => "bed",
                                ObjectKind::BerryBush => "berrybush",
                            };
                            use std::fmt::Write as _;
                            let _ = write!(
                                json,
                                r#"{{"col":{col},"row":{row},"kind":"{kind_str}","charges":{charges}}}"#
                            );
                        }
                        MapTileObject::ConstructionSite(kind) => {
                            let kind_str = match kind {
                                ObjectKind::Wall => "wall",
                                ObjectKind::Bed => "bed",
                                ObjectKind::BerryBush => "berrybush",
                            };
                            use std::fmt::Write as _;
                            let _ = write!(
                                json,
                                r#"{{"col":{col},"row":{row},"kind":"ConstructionSite","underlying":"{kind_str}"}}"#
                            );
                        }
                    }
                }
            }
        }
        json.push(']');
        json
    }

    #[wasm_bindgen(js_name = getConstructionProgress)]
    pub fn get_construction_progress(&self) -> String {
        let queue = self.world.resource::<ConstructionQueue>();
        let mut json = String::from("[");
        for (i, job) in queue.jobs.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            let kind_str = match job.kind {
                ObjectKind::Wall => "wall",
                ObjectKind::Bed => "bed",
                ObjectKind::BerryBush => "berrybush",
            };
            use std::fmt::Write as _;
            let _ = write!(
                json,
                r#"{{"col":{},"row":{},"progress":{},"maxProgress":{},"kind":"{}"}}"#,
                job.col, job.row, job.progress, job.max_progress, kind_str
            );
        }
        json.push(']');
        json
    }

    #[wasm_bindgen(js_name = getUnitPositions)]
    pub fn get_unit_positions(&mut self) -> String {
        let mut units: Vec<(u32, f32, f32)> = self
            .world
            .query::<(&UnitId, &Position)>()
            .iter(&self.world)
            .map(|(id, pos)| (id.0, pos.x, pos.y))
            .collect();
        units.sort_by_key(|(id, _, _)| *id);

        let mut json = String::from("[");
        for (index, (id, x, y)) in units.iter().enumerate() {
            if index > 0 {
                json.push(',');
            }
            use std::fmt::Write as _;
            let _ = write!(json, r#"{{"id":{id},"x":{x},"y":{y}}}"#);
        }
        json.push(']');
        json
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Path, Position};

    type UnitPathSnapshot = ((f32, f32), Vec<(f32, f32)>);

    #[test]
    fn initial_world_has_three_unique_charged_bushes_for_fixed_seeds() {
        for seed in [0, 1, 2, 42, 12345, u64::MAX] {
            let game = create_game_world(3, seed);
            let map = game.world.resource::<MapObjects>();
            let tile_map = game.world.resource::<TileMapResource>();
            let bushes: Vec<usize> = map
                .tiles
                .iter()
                .enumerate()
                .filter_map(|(idx, object)| match object {
                    Some(MapTileObject::FoodSource(ObjectKind::BerryBush, 5)) => Some(idx),
                    _ => None,
                })
                .collect();

            assert_eq!(bushes.len(), 3, "seed {seed} should create three bushes");
            assert_eq!(
                bushes
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
                3,
                "seed {seed} should not collide bushes"
            );
            for idx in bushes {
                let col = idx as u32 % tile_map.cols;
                let row = idx as u32 / tile_map.cols;
                assert!(tile_map.is_walkable(col, row));
            }
        }
    }

    #[test]
    fn spawned_units_have_non_empty_walkable_paths() {
        let mut game = create_game_world(3, 42);
        let paths: Vec<UnitPathSnapshot> = {
            let mut query = game.world.query::<(&Position, &Path)>();
            query
                .iter(&game.world)
                .map(|(position, path)| ((position.x, position.y), path.waypoints.clone()))
                .collect()
        };
        let tile_map = game.world.resource::<TileMapResource>();

        assert_eq!(paths.len(), 3);
        for ((x, y), waypoints) in paths {
            assert!(!waypoints.is_empty());
            let mut previous = tile_map.world_to_tile(x, y);
            assert!(tile_map.is_walkable(previous.0, previous.1));
            for (waypoint_x, waypoint_y) in waypoints {
                let current = tile_map.world_to_tile(waypoint_x, waypoint_y);
                assert!(tile_map.is_walkable(current.0, current.1));
                assert!(
                    previous.0.abs_diff(current.0) + previous.1.abs_diff(current.1) <= 1,
                    "path must remain 4-connected"
                );
                previous = current;
            }
        }
    }
}
