use crate::components::{
    AssignedJob, Energy, HungryDebuff, NeedKind, NeedsPlan, Path, Position, Satiation, Speed,
    TiredDebuff, UnitId,
};
use crate::events::{BuildRequest, Hungry, Rested, Sated, TargetReached, Tired};
use crate::pathfinding::{astar, random_reachable_path, tile_path_to_waypoints};
use crate::resources::{
    ConstructionJob, ConstructionQueue, DeltaTime, MapObjects, MapTileObject, ObjectKind,
    SimulationRng, TileMapResource,
};
use crate::world::FIELD_HEIGHT;
use crate::world::FIELD_WIDTH;
use bevy_ecs::prelude::*;

pub const DEFAULT_SPEED: f32 = 3.75;
pub const ARRIVAL_THRESHOLD: f32 = 0.125;
pub const MAX_DELTA_MS: f32 = 200.0;

pub const HUNGER_RATE: f32 = 0.8;
pub const FATIGUE_RATE: f32 = 0.4;
pub const EAT_RATE: f32 = 15.0;
pub const SLEEP_RATE: f32 = 7.0;
pub const HUNGER_THRESHOLD: f32 = 30.0;
pub const ENERGY_THRESHOLD: f32 = 30.0;
pub const SATIATED_THRESHOLD: f32 = 90.0;
pub const RESTED_THRESHOLD: f32 = 90.0;
pub const ARRIVAL_TILE_THRESHOLD: f32 = 0.5;

pub const BUILD_SPEED: f32 = 15.0;
pub const MAX_WORKERS_PER_JOB: usize = 2;

type JobAssignment = (Entity, usize, Vec<(f32, f32)>);
type CandidateAssignment = (usize, f32, Vec<(f32, f32)>);

pub fn needs_accrual(
    time: Res<DeltaTime>,
    mut query: Query<(Option<&mut Satiation>, Option<&mut Energy>)>,
) {
    let dt = time.0;
    for (satiation, energy) in query.iter_mut() {
        if let Some(mut s) = satiation {
            s.0 = (s.0 - HUNGER_RATE * dt).max(0.0);
        }
        if let Some(mut e) = energy {
            e.0 = (e.0 - FATIGUE_RATE * dt).max(0.0);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn needs_event_check(
    mut hungry_writer: MessageWriter<Hungry>,
    mut tired_writer: MessageWriter<Tired>,
    mut sated_writer: MessageWriter<Sated>,
    mut rested_writer: MessageWriter<Rested>,
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &Satiation,
            &Energy,
            Option<&HungryDebuff>,
            Option<&TiredDebuff>,
        ),
        With<UnitId>,
    >,
) {
    for (entity, satiation, energy, hungry_debuff, tired_debuff) in query.iter() {
        if satiation.0 < HUNGER_THRESHOLD && hungry_debuff.is_none() {
            hungry_writer.write(Hungry(entity));
            commands.entity(entity).insert(HungryDebuff);
        }
        if energy.0 < ENERGY_THRESHOLD && tired_debuff.is_none() {
            tired_writer.write(Tired(entity));
            commands.entity(entity).insert(TiredDebuff);
        }
        if satiation.0 > SATIATED_THRESHOLD && hungry_debuff.is_some() {
            sated_writer.write(Sated(entity));
            commands.entity(entity).remove::<HungryDebuff>();
        }
        if energy.0 > RESTED_THRESHOLD && tired_debuff.is_some() {
            rested_writer.write(Rested(entity));
            commands.entity(entity).remove::<TiredDebuff>();
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn needs_decision(
    mut hungry_reader: MessageReader<Hungry>,
    mut tired_reader: MessageReader<Tired>,
    mut sated_reader: MessageReader<Sated>,
    mut rested_reader: MessageReader<Rested>,
    mut commands: Commands,
    mut map_objects: ResMut<MapObjects>,
    tile_map: Res<TileMapResource>,
    debuffs: Query<
        (
            Entity,
            &Position,
            Option<&HungryDebuff>,
            Option<&TiredDebuff>,
        ),
        With<UnitId>,
    >,
    needs_query: Query<
        (
            Entity,
            &Position,
            Option<&HungryDebuff>,
            Option<&TiredDebuff>,
            Option<&NeedsPlan>,
        ),
        With<UnitId>,
    >,
    plans: Query<&NeedsPlan>,
) {
    for Hungry(entity) in hungry_reader.read() {
        if let Ok(pos) = debuffs.get(*entity).map(|(_, pos, _, _)| pos)
            && let Some(target) = find_nearest_object(
                &map_objects,
                &tile_map,
                (pos.x, pos.y),
                ObjectKind::BerryBush,
            )
        {
            commands.entity(*entity).insert(NeedsPlan {
                kind: NeedKind::Eat,
                target,
            });
        }
    }

    for Tired(entity) in tired_reader.read() {
        if let Ok(pos) = debuffs.get(*entity).map(|(_, pos, _, _)| pos)
            && let Some(target) =
                find_nearest_object(&map_objects, &tile_map, (pos.x, pos.y), ObjectKind::Bed)
        {
            commands.entity(*entity).insert(NeedsPlan {
                kind: NeedKind::Sleep,
                target,
            });
        }
    }

    for Sated(entity) in sated_reader.read() {
        if let Ok(plan) = plans.get(*entity)
            && plan.kind == NeedKind::Eat
        {
            let (tx, ty) = plan.target;
            let col = tx.floor() as u32;
            let row = ty.floor() as u32;
            if col < tile_map.cols && row < tile_map.rows {
                let idx = (row * tile_map.cols + col) as usize;
                if idx < map_objects.tiles.len()
                    && let Some(MapTileObject::FoodSource(ObjectKind::BerryBush, ref mut charges)) =
                        map_objects.tiles[idx]
                {
                    if *charges > 0 {
                        *charges -= 1;
                    }
                    if *charges == 0 {
                        map_objects.tiles[idx] = None;
                    }
                }
            }
        }
        commands.entity(*entity).remove::<NeedsPlan>();
        if let Ok((_, pos, _, tired_debuff)) = debuffs.get(*entity)
            && tired_debuff.is_some()
            && let Some(target) =
                find_nearest_object(&map_objects, &tile_map, (pos.x, pos.y), ObjectKind::Bed)
        {
            commands.entity(*entity).insert(NeedsPlan {
                kind: NeedKind::Sleep,
                target,
            });
        }
    }

    for Rested(entity) in rested_reader.read() {
        commands.entity(*entity).remove::<NeedsPlan>();
        if let Ok((_, pos, hungry_debuff, _)) = debuffs.get(*entity)
            && hungry_debuff.is_some()
            && let Some(target) = find_nearest_object(
                &map_objects,
                &tile_map,
                (pos.x, pos.y),
                ObjectKind::BerryBush,
            )
        {
            commands.entity(*entity).insert(NeedsPlan {
                kind: NeedKind::Eat,
                target,
            });
        }
    }

    for (entity, pos, hungry_debuff, tired_debuff, plan) in needs_query.iter() {
        if plan.is_some() {
            continue;
        }
        if tired_debuff.is_some()
            && let Some(target) =
                find_nearest_object(&map_objects, &tile_map, (pos.x, pos.y), ObjectKind::Bed)
        {
            commands.entity(entity).insert(NeedsPlan {
                kind: NeedKind::Sleep,
                target,
            });
            continue;
        }
        if hungry_debuff.is_some()
            && let Some(target) = find_nearest_object(
                &map_objects,
                &tile_map,
                (pos.x, pos.y),
                ObjectKind::BerryBush,
            )
        {
            commands.entity(entity).insert(NeedsPlan {
                kind: NeedKind::Eat,
                target,
            });
        }
    }
}

pub fn execute_needs_plan(
    time: Res<DeltaTime>,
    mut query: Query<(
        &NeedsPlan,
        &mut Path,
        &Position,
        &mut Satiation,
        &mut Energy,
    )>,
    tile_map: Res<TileMapResource>,
) {
    let dt = time.0;
    for (plan, mut path, pos, mut satiation, mut energy) in query.iter_mut() {
        let (tx, ty) = plan.target;
        let dx = tx - pos.x;
        let dy = ty - pos.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist > ARRIVAL_TILE_THRESHOLD {
            let (start_col, start_row) = tile_map.world_to_tile(pos.x, pos.y);
            let (goal_col, goal_row) = tile_map.world_to_tile(tx, ty);
            if let Some(tile_path) = astar((start_col, start_row), (goal_col, goal_row), &tile_map)
            {
                let waypoints: Vec<(f32, f32)> = tile_path
                    .iter()
                    .map(|&(c, r)| tile_map.tile_to_world(c, r))
                    .collect();
                path.waypoints = waypoints;
            }
        } else {
            path.waypoints.clear();
            match plan.kind {
                NeedKind::Eat => {
                    satiation.0 = (satiation.0 + EAT_RATE * dt).min(100.0);
                }
                NeedKind::Sleep => {
                    energy.0 = (energy.0 + SLEEP_RATE * dt).min(100.0);
                }
            }
        }
    }
}

pub fn release_job_on_needs(
    mut construction_queue: ResMut<ConstructionQueue>,
    query: Query<(Entity, &AssignedJob), With<NeedsPlan>>,
    mut commands: Commands,
) {
    for (entity, _assigned_job) in query.iter() {
        for job in construction_queue.jobs.iter_mut() {
            job.assigned_units.retain(|e| *e != entity);
        }
        commands.entity(entity).remove::<AssignedJob>();
    }
}

fn construction_tile_index(
    tile_map: &TileMapResource,
    map_objects: &MapObjects,
    col: u32,
    row: u32,
) -> Option<usize> {
    if col >= tile_map.cols || row >= tile_map.rows || !tile_map.is_walkable(col, row) {
        return None;
    }

    let idx = (row * tile_map.cols + col) as usize;
    if idx >= map_objects.tiles.len() || map_objects.tiles[idx].is_some() {
        return None;
    }

    Some(idx)
}

pub fn construction_system(
    mut reader: MessageReader<BuildRequest>,
    mut map_objects: ResMut<MapObjects>,
    tile_map: Res<TileMapResource>,
    mut construction_queue: ResMut<ConstructionQueue>,
) {
    for event in reader.read() {
        let Some(idx) = construction_tile_index(&tile_map, &map_objects, event.col, event.row)
        else {
            continue;
        };

        map_objects.tiles[idx] = Some(MapTileObject::ConstructionSite(event.kind));

        let max_progress = match event.kind {
            ObjectKind::Wall => 50.0,
            ObjectKind::Bed => 80.0,
            ObjectKind::BerryBush => 40.0,
        };

        construction_queue.jobs.push(ConstructionJob {
            col: event.col,
            row: event.row,
            kind: event.kind,
            progress: 0.0,
            max_progress,
            assigned_units: Vec::new(),
        });
    }
}

pub fn job_assignment_system(
    mut construction_queue: ResMut<ConstructionQueue>,
    tile_map: Res<TileMapResource>,
    query: Query<(Entity, &Position, Option<&NeedsPlan>, Option<&AssignedJob>)>,
    mut commands: Commands,
) {
    let mut assignments: Vec<JobAssignment> = Vec::new();

    for (entity, pos, needs_plan, assigned_job) in query.iter() {
        if needs_plan.is_some() || assigned_job.is_some() {
            continue;
        }

        let (start_col, start_row) = tile_map.world_to_tile(pos.x, pos.y);
        let mut best_assignment: Option<CandidateAssignment> = None;
        let mut best_dist = f32::MAX;

        for (i, job) in construction_queue.jobs.iter().enumerate() {
            if job.assigned_units.len() >= MAX_WORKERS_PER_JOB {
                continue;
            }

            let Some(tile_path) = astar((start_col, start_row), (job.col, job.row), &tile_map)
            else {
                continue;
            };

            let (jx, jy) = tile_map.tile_to_world(job.col, job.row);
            let dx = jx - pos.x;
            let dy = jy - pos.y;
            let dist = dx * dx + dy * dy;
            if dist < best_dist {
                best_dist = dist;
                best_assignment = Some((i, dist, tile_path_to_waypoints(&tile_path, &tile_map)));
            }
        }

        if let Some((job_idx, _, waypoints)) = best_assignment {
            assignments.push((entity, job_idx, waypoints));
        }
    }

    for (entity, job_idx, waypoints) in assignments {
        let job = &mut construction_queue.jobs[job_idx];
        if job.assigned_units.len() < MAX_WORKERS_PER_JOB {
            job.assigned_units.push(entity);
            commands.entity(entity).insert(AssignedJob);
            commands.entity(entity).insert(Path { waypoints });
        }
    }
}

pub fn construction_progress_system(
    mut construction_queue: ResMut<ConstructionQueue>,
    mut map_objects: ResMut<MapObjects>,
    mut tile_map: ResMut<TileMapResource>,
    time: Res<DeltaTime>,
    positions: Query<&Position>,
    mut commands: Commands,
) {
    let dt = time.0;
    if dt == 0.0 {
        return;
    }

    let mut completed: Vec<usize> = Vec::new();

    for (i, job) in construction_queue.jobs.iter_mut().enumerate() {
        let (site_cx, site_cy) = tile_map.tile_to_world(job.col, job.row);

        let mut workers_at_site = 0u32;
        for &unit_entity in &job.assigned_units {
            if let Ok(pos) = positions.get(unit_entity) {
                let dx = pos.x - site_cx;
                let dy = pos.y - site_cy;
                if dx * dx + dy * dy < ARRIVAL_TILE_THRESHOLD * ARRIVAL_TILE_THRESHOLD {
                    workers_at_site += 1;
                }
            }
        }

        if workers_at_site == 0 {
            continue;
        }
        job.progress += BUILD_SPEED * workers_at_site as f32 * dt;

        if job.progress >= job.max_progress {
            completed.push(i);
        }
    }

    for i in completed.into_iter().rev() {
        let assigned_entities: Vec<Entity> = construction_queue.jobs[i].assigned_units.clone();
        let job = &construction_queue.jobs[i];
        let idx = (job.row * tile_map.cols + job.col) as usize;

        if idx < map_objects.tiles.len() {
            if job.kind == ObjectKind::BerryBush {
                map_objects.tiles[idx] = Some(MapTileObject::FoodSource(ObjectKind::BerryBush, 5));
            } else {
                map_objects.tiles[idx] = Some(MapTileObject::Building(job.kind));
            }
        }

        if job.kind == ObjectKind::Wall && idx < tile_map.tiles.len() {
            tile_map.tiles[idx] = false;
        }

        construction_queue.jobs.swap_remove(i);

        for entity in assigned_entities {
            commands.entity(entity).remove::<AssignedJob>();
        }
    }
}

fn find_nearest_object(
    map_objects: &MapObjects,
    tile_map: &TileMapResource,
    origin: (f32, f32),
    kind: ObjectKind,
) -> Option<(f32, f32)> {
    let mut best_dist = f32::MAX;
    let mut best_pos = None;
    for row in 0..tile_map.rows {
        for col in 0..tile_map.cols {
            let idx = (row * tile_map.cols + col) as usize;
            if idx >= map_objects.tiles.len() {
                continue;
            }
            if !tile_map.is_walkable(col, row) {
                continue;
            }
            match map_objects.tiles[idx] {
                Some(MapTileObject::Building(k)) if k == kind => {}
                Some(MapTileObject::FoodSource(k, charges)) if k == kind && charges > 0 => {}
                _ => continue,
            }
            let (cx, cy) = tile_map.tile_to_world(col, row);
            let d = (cx - origin.0) * (cx - origin.0) + (cy - origin.1) * (cy - origin.1);
            if d < best_dist {
                best_dist = d;
                best_pos = Some((cx, cy));
            }
        }
    }
    best_pos
}

fn waypoints_are_walkable(
    position: &Position,
    waypoints: &[(f32, f32)],
    tile_map: &TileMapResource,
) -> bool {
    let mut previous = tile_map.world_to_tile(position.x, position.y);
    if !tile_map.is_walkable(previous.0, previous.1) {
        return false;
    }

    for &(x, y) in waypoints {
        let current = tile_map.world_to_tile(x, y);
        if !tile_map.is_walkable(current.0, current.1) {
            return false;
        }
        let distance = previous.0.abs_diff(current.0) + previous.1.abs_diff(current.1);
        if distance > 1 {
            return false;
        }
        previous = current;
    }

    true
}

pub fn move_along_path(
    mut writer: MessageWriter<TargetReached>,
    time: Res<DeltaTime>,
    tile_map: Res<TileMapResource>,
    mut query: Query<(Entity, &mut Position, &mut Path, &Speed)>,
) {
    let delta_secs = time.0;

    for (entity, mut pos, mut path, speed) in query.iter_mut() {
        if path.waypoints.is_empty() {
            writer.write(TargetReached { entity });
            continue;
        }

        if !waypoints_are_walkable(&pos, &path.waypoints, &tile_map) {
            path.waypoints.clear();
            writer.write(TargetReached { entity });
            continue;
        }

        while !path.waypoints.is_empty() {
            let (wx, wy) = path.waypoints[0];
            let dx = wx - pos.x;
            let dy = wy - pos.y;
            if dx * dx + dy * dy < ARRIVAL_THRESHOLD * ARRIVAL_THRESHOLD {
                path.waypoints.remove(0);
            } else {
                break;
            }
        }
        if path.waypoints.is_empty() {
            writer.write(TargetReached { entity });
            continue;
        }

        let target_x = path.waypoints[0].0;
        let target_y = path.waypoints[0].1;
        let dx = target_x - pos.x;
        let dy = target_y - pos.y;
        let dist_sq = dx * dx + dy * dy;

        if dist_sq < ARRIVAL_THRESHOLD * ARRIVAL_THRESHOLD {
            path.waypoints.remove(0);
            if path.waypoints.is_empty() {
                writer.write(TargetReached { entity });
            }
            continue;
        }

        let dist = dist_sq.sqrt();
        let step = (speed.0 * delta_secs).min(dist);
        pos.x += (dx / dist) * step;
        pos.y += (dy / dist) * step;

        pos.x = pos.x.clamp(0.0, FIELD_WIDTH);
        pos.y = pos.y.clamp(0.0, FIELD_HEIGHT);
    }
}

pub fn find_path_action(
    mut reader: MessageReader<TargetReached>,
    mut paths: Query<(&Position, &mut Path)>,
    plans: Query<&NeedsPlan>,
    jobs: Query<&AssignedJob>,
    tile_map: Res<TileMapResource>,
    mut rng: ResMut<SimulationRng>,
) {
    for event in reader.read() {
        if plans.get(event.entity).is_ok() {
            continue;
        }
        if jobs.get(event.entity).is_ok() {
            continue;
        }
        let Ok((pos, mut path)) = paths.get_mut(event.entity) else {
            continue;
        };

        let (start_col, start_row) = tile_map.world_to_tile(pos.x, pos.y);

        if let Some(tile_path) =
            random_reachable_path((start_col, start_row), &tile_map, &mut rng.0)
        {
            path.waypoints = tile_path_to_waypoints(&tile_path, &tile_map);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::ConstructionJob;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn make_world(cols: u32, rows: u32) -> World {
        let mut world = World::new();
        world.init_resource::<Messages<BuildRequest>>();
        world.init_resource::<Messages<Hungry>>();
        world.init_resource::<Messages<Tired>>();
        world.init_resource::<Messages<Sated>>();
        world.init_resource::<Messages<Rested>>();
        world.init_resource::<Messages<TargetReached>>();
        world.insert_resource(TileMapResource::new(42));
        world.insert_resource(MapObjects::new(cols, rows));
        world.insert_resource(ConstructionQueue::new());
        world.insert_resource(SimulationRng(StdRng::seed_from_u64(42)));
        world
    }

    #[test]
    fn construction_system_creates_site_and_job() {
        let mut world = make_world(25, 19);

        world
            .resource_mut::<Messages<BuildRequest>>()
            .write(BuildRequest {
                col: 5,
                row: 7,
                kind: ObjectKind::Wall,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(construction_system);
        schedule.run(&mut world);

        let idx = (7 * 25 + 5) as usize;
        let map = world.resource::<MapObjects>();
        assert!(
            matches!(
                &map.tiles[idx],
                Some(MapTileObject::ConstructionSite(ObjectKind::Wall))
            ),
            "expected ConstructionSite(Wall) at (5,7)"
        );

        let queue = world.resource::<ConstructionQueue>();
        assert_eq!(queue.jobs.len(), 1);
        assert_eq!(queue.jobs[0].col, 5);
        assert_eq!(queue.jobs[0].row, 7);
        assert_eq!(queue.jobs[0].kind, ObjectKind::Wall);
        assert_eq!(queue.jobs[0].progress, 0.0);
        assert_eq!(queue.jobs[0].max_progress, 50.0);
    }

    #[test]
    fn construction_system_ignores_occupied_tile() {
        let mut world = make_world(25, 19);
        let idx = (3 * 25 + 3) as usize;
        world.resource_mut::<MapObjects>().tiles[idx] =
            Some(MapTileObject::Building(ObjectKind::Bed));

        world
            .resource_mut::<Messages<BuildRequest>>()
            .write(BuildRequest {
                col: 3,
                row: 3,
                kind: ObjectKind::Wall,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(construction_system);
        schedule.run(&mut world);

        let map = world.resource::<MapObjects>();
        assert!(
            matches!(
                &map.tiles[idx],
                Some(MapTileObject::Building(ObjectKind::Bed))
            ),
            "tile should remain unchanged"
        );
        let queue = world.resource::<ConstructionQueue>();
        assert!(queue.jobs.is_empty());
    }

    #[test]
    fn construction_system_ignores_all_kinds_on_occupied_tile() {
        for kind in [ObjectKind::Wall, ObjectKind::Bed, ObjectKind::BerryBush] {
            let mut world = make_world(25, 19);
            let idx = (3 * 25 + 3) as usize;
            world.resource_mut::<MapObjects>().tiles[idx] =
                Some(MapTileObject::Building(ObjectKind::Bed));
            world
                .resource_mut::<Messages<BuildRequest>>()
                .write(BuildRequest {
                    col: 3,
                    row: 3,
                    kind,
                });

            let mut schedule = Schedule::default();
            schedule.add_systems(construction_system);
            schedule.run(&mut world);

            assert!(matches!(
                &world.resource::<MapObjects>().tiles[idx],
                Some(MapTileObject::Building(ObjectKind::Bed))
            ));
            assert!(world.resource::<ConstructionQueue>().jobs.is_empty());
        }
    }

    #[test]
    fn construction_system_ignores_blocked_tile() {
        let mut world = make_world(25, 19);
        let idx = (4 * 25 + 4) as usize;
        world.resource_mut::<TileMapResource>().tiles[idx] = false;

        world
            .resource_mut::<Messages<BuildRequest>>()
            .write(BuildRequest {
                col: 4,
                row: 4,
                kind: ObjectKind::Wall,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(construction_system);
        schedule.run(&mut world);

        let map = world.resource::<MapObjects>();
        assert!(
            map.tiles[idx].is_none(),
            "blocked tile should not get a wall"
        );
        let queue = world.resource::<ConstructionQueue>();
        assert!(queue.jobs.is_empty());
    }

    #[test]
    fn construction_system_ignores_all_kinds_on_blocked_tile() {
        for kind in [ObjectKind::Wall, ObjectKind::Bed, ObjectKind::BerryBush] {
            let mut world = make_world(25, 19);
            let idx = (4 * 25 + 4) as usize;
            world.resource_mut::<TileMapResource>().tiles[idx] = false;
            world
                .resource_mut::<Messages<BuildRequest>>()
                .write(BuildRequest {
                    col: 4,
                    row: 4,
                    kind,
                });

            let mut schedule = Schedule::default();
            schedule.add_systems(construction_system);
            schedule.run(&mut world);

            assert!(world.resource::<MapObjects>().tiles[idx].is_none());
            assert!(world.resource::<ConstructionQueue>().jobs.is_empty());
        }
    }

    #[test]
    fn job_assignment_assigns_free_colonist() {
        let mut world = make_world(25, 19);

        let colonist = world
            .spawn((
                UnitId(0),
                Position { x: 5.5, y: 5.5 },
                Path {
                    waypoints: Vec::new(),
                },
                Speed(3.75),
                Satiation(100.0),
                Energy(100.0),
            ))
            .id();

        world
            .resource_mut::<ConstructionQueue>()
            .jobs
            .push(ConstructionJob {
                col: 5,
                row: 5,
                kind: ObjectKind::Wall,
                progress: 0.0,
                max_progress: 50.0,
                assigned_units: Vec::new(),
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(job_assignment_system);
        schedule.run(&mut world);

        let queue = world.resource::<ConstructionQueue>();
        assert_eq!(queue.jobs[0].assigned_units.len(), 1);
        assert_eq!(queue.jobs[0].assigned_units[0], colonist);

        assert!(world.entity(colonist).contains::<AssignedJob>());
    }

    #[test]
    fn job_assignment_skips_colonist_with_needs() {
        let mut world = make_world(25, 19);

        let _colonist = world
            .spawn((
                UnitId(0),
                Position { x: 5.5, y: 5.5 },
                Path {
                    waypoints: Vec::new(),
                },
                NeedsPlan {
                    kind: NeedKind::Eat,
                    target: (10.0, 10.0),
                },
            ))
            .id();

        world
            .resource_mut::<ConstructionQueue>()
            .jobs
            .push(ConstructionJob {
                col: 5,
                row: 5,
                kind: ObjectKind::Wall,
                progress: 0.0,
                max_progress: 50.0,
                assigned_units: Vec::new(),
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(job_assignment_system);
        schedule.run(&mut world);

        let queue = world.resource::<ConstructionQueue>();
        assert!(queue.jobs[0].assigned_units.is_empty());
    }

    #[test]
    fn job_assignment_skips_colonist_with_existing_job() {
        let mut world = make_world(25, 19);

        let _colonist = world
            .spawn((
                UnitId(0),
                Position { x: 5.5, y: 5.5 },
                Path {
                    waypoints: Vec::new(),
                },
                AssignedJob,
            ))
            .id();

        world
            .resource_mut::<ConstructionQueue>()
            .jobs
            .push(ConstructionJob {
                col: 5,
                row: 5,
                kind: ObjectKind::Wall,
                progress: 0.0,
                max_progress: 50.0,
                assigned_units: Vec::new(),
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(job_assignment_system);
        schedule.run(&mut world);

        let queue = world.resource::<ConstructionQueue>();
        assert!(queue.jobs[0].assigned_units.is_empty());
    }

    #[test]
    fn release_job_on_need_removes_assigned() {
        let mut world = make_world(25, 19);

        let colonist = world
            .spawn((
                UnitId(0),
                Position { x: 5.5, y: 5.5 },
                NeedsPlan {
                    kind: NeedKind::Eat,
                    target: (10.0, 10.0),
                },
                AssignedJob,
            ))
            .id();

        world
            .resource_mut::<ConstructionQueue>()
            .jobs
            .push(ConstructionJob {
                col: 5,
                row: 5,
                kind: ObjectKind::Wall,
                progress: 0.0,
                max_progress: 50.0,
                assigned_units: vec![colonist],
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(release_job_on_needs);
        schedule.run(&mut world);

        let queue = world.resource::<ConstructionQueue>();
        assert!(
            queue.jobs[0].assigned_units.is_empty(),
            "colonist should be removed from job"
        );

        assert!(
            !world.entity(colonist).contains::<AssignedJob>(),
            "AssignedJob should be removed from entity"
        );
    }

    #[test]
    fn progress_accumulates_with_worker_at_site() {
        let mut world = make_world(25, 19);
        let tile_map = world.resource::<TileMapResource>();
        let (cx, cy) = tile_map.tile_to_world(5, 5);

        let _colonist = world.spawn((UnitId(0), Position { x: cx, y: cy })).id();

        world
            .resource_mut::<ConstructionQueue>()
            .jobs
            .push(ConstructionJob {
                col: 5,
                row: 5,
                kind: ObjectKind::Wall,
                progress: 0.0,
                max_progress: 50.0,
                assigned_units: vec![_colonist],
            });
        world.insert_resource(DeltaTime(1.0));

        let mut schedule = Schedule::default();
        schedule.add_systems(construction_progress_system);
        schedule.run(&mut world);

        let queue = world.resource::<ConstructionQueue>();
        assert_eq!(
            queue.jobs[0].progress,
            BUILD_SPEED * 1.0,
            "progress should increase by BUILD_SPEED * dt for 1 worker"
        );
    }

    #[test]
    fn progress_requires_proximity() {
        let mut world = make_world(25, 19);

        let _colonist = world
            .spawn((UnitId(0), Position { x: 100.0, y: 100.0 }))
            .id();

        world
            .resource_mut::<ConstructionQueue>()
            .jobs
            .push(ConstructionJob {
                col: 5,
                row: 5,
                kind: ObjectKind::Wall,
                progress: 0.0,
                max_progress: 50.0,
                assigned_units: vec![_colonist],
            });
        world.insert_resource(DeltaTime(1.0));

        let mut schedule = Schedule::default();
        schedule.add_systems(construction_progress_system);
        schedule.run(&mut world);

        let queue = world.resource::<ConstructionQueue>();
        assert_eq!(
            queue.jobs[0].progress, 0.0,
            "distant worker should not contribute to progress"
        );
    }

    #[test]
    fn multiple_workers_speed_up_progress() {
        let mut world = make_world(25, 19);
        let tile_map = world.resource::<TileMapResource>();
        let (cx, cy) = tile_map.tile_to_world(5, 5);

        let w1 = world.spawn((UnitId(0), Position { x: cx, y: cy })).id();
        let w2 = world.spawn((UnitId(1), Position { x: cx, y: cy })).id();

        world
            .resource_mut::<ConstructionQueue>()
            .jobs
            .push(ConstructionJob {
                col: 5,
                row: 5,
                kind: ObjectKind::Wall,
                progress: 0.0,
                max_progress: 100.0,
                assigned_units: vec![w1, w2],
            });
        world.insert_resource(DeltaTime(1.0));

        let mut schedule = Schedule::default();
        schedule.add_systems(construction_progress_system);
        schedule.run(&mut world);

        let queue = world.resource::<ConstructionQueue>();
        assert_eq!(
            queue.jobs[0].progress,
            BUILD_SPEED * 2.0,
            "2 workers should give 2x speed"
        );
    }

    #[test]
    fn no_workers_no_progress() {
        let mut world = make_world(25, 19);

        world
            .resource_mut::<ConstructionQueue>()
            .jobs
            .push(ConstructionJob {
                col: 5,
                row: 5,
                kind: ObjectKind::Wall,
                progress: 0.0,
                max_progress: 50.0,
                assigned_units: Vec::new(),
            });
        world.insert_resource(DeltaTime(1.0));

        let mut schedule = Schedule::default();
        schedule.add_systems(construction_progress_system);
        schedule.run(&mut world);

        let queue = world.resource::<ConstructionQueue>();
        assert_eq!(
            queue.jobs[0].progress, 0.0,
            "no workers should mean no progress"
        );
    }

    #[test]
    fn construction_completes_and_becomes_building() {
        let mut world = make_world(25, 19);
        let tile_map = world.resource::<TileMapResource>();
        let (cx, cy) = tile_map.tile_to_world(3, 4);

        let _colonist = world.spawn((UnitId(0), Position { x: cx, y: cy })).id();

        let idx = (4 * 25 + 3) as usize;
        world.resource_mut::<MapObjects>().tiles[idx] =
            Some(MapTileObject::ConstructionSite(ObjectKind::Bed));

        world
            .resource_mut::<ConstructionQueue>()
            .jobs
            .push(ConstructionJob {
                col: 3,
                row: 4,
                kind: ObjectKind::Bed,
                progress: 79.0,
                max_progress: 80.0,
                assigned_units: vec![_colonist],
            });
        world.insert_resource(DeltaTime(1.0));

        let mut schedule = Schedule::default();
        schedule.add_systems(construction_progress_system);
        schedule.run(&mut world);

        let map = world.resource::<MapObjects>();
        assert!(
            matches!(
                &map.tiles[idx],
                Some(MapTileObject::Building(ObjectKind::Bed))
            ),
            "site should become a finished building"
        );

        let queue = world.resource::<ConstructionQueue>();
        assert!(queue.jobs.is_empty(), "completed job should be removed");
    }

    #[test]
    fn wall_blocks_pathfinding_after_completion() {
        let mut world = make_world(25, 19);
        let tile_map = world.resource::<TileMapResource>();
        let (cx, cy) = tile_map.tile_to_world(7, 8);

        let _colonist = world.spawn((UnitId(0), Position { x: cx, y: cy })).id();

        world
            .resource_mut::<ConstructionQueue>()
            .jobs
            .push(ConstructionJob {
                col: 7,
                row: 8,
                kind: ObjectKind::Wall,
                progress: 49.0,
                max_progress: 50.0,
                assigned_units: vec![_colonist],
            });
        world.insert_resource(DeltaTime(1.0));

        let mut schedule = Schedule::default();
        schedule.add_systems(construction_progress_system);
        schedule.run(&mut world);

        let tile_map = world.resource::<TileMapResource>();
        assert!(
            !tile_map.is_walkable(7, 8),
            "wall tile should be blocked after construction"
        );
    }

    #[test]
    fn berry_bush_completion_keeps_tile_walkable() {
        let mut world = make_world(25, 19);
        let tile_map = world.resource::<TileMapResource>();
        let (cx, cy) = tile_map.tile_to_world(3, 4);
        let colonist = world.spawn((UnitId(0), Position { x: cx, y: cy })).id();
        let idx = (4 * 25 + 3) as usize;

        world.resource_mut::<MapObjects>().tiles[idx] =
            Some(MapTileObject::ConstructionSite(ObjectKind::BerryBush));
        world
            .resource_mut::<ConstructionQueue>()
            .jobs
            .push(ConstructionJob {
                col: 3,
                row: 4,
                kind: ObjectKind::BerryBush,
                progress: 39.0,
                max_progress: 40.0,
                assigned_units: vec![colonist],
            });
        world.insert_resource(DeltaTime(1.0));

        let mut schedule = Schedule::default();
        schedule.add_systems(construction_progress_system);
        schedule.run(&mut world);

        assert!(matches!(
            &world.resource::<MapObjects>().tiles[idx],
            Some(MapTileObject::FoodSource(ObjectKind::BerryBush, 5))
        ));
        assert!(world.resource::<TileMapResource>().is_walkable(3, 4));
    }

    #[test]
    fn nearest_object_is_selected_relative_to_unit_position() {
        let mut tile_map = TileMapResource::new(42);
        tile_map.tiles.fill(true);
        let mut map_objects = MapObjects::new(tile_map.cols, tile_map.rows);
        let first = (tile_map.cols + 1) as usize;
        let second = (15 * tile_map.cols + 20) as usize;
        map_objects.tiles[first] = Some(MapTileObject::FoodSource(ObjectKind::BerryBush, 5));
        map_objects.tiles[second] = Some(MapTileObject::FoodSource(ObjectKind::BerryBush, 5));

        assert_eq!(
            find_nearest_object(&map_objects, &tile_map, (0.5, 0.5), ObjectKind::BerryBush,),
            Some((1.5, 1.5))
        );
        assert_eq!(
            find_nearest_object(&map_objects, &tile_map, (20.5, 15.5), ObjectKind::BerryBush,),
            Some((20.5, 15.5))
        );
    }

    #[test]
    fn no_reachable_target_does_not_create_direct_fallback() {
        let mut world = make_world(25, 19);
        let start_idx = 5 * 25 + 5;
        world.resource_mut::<TileMapResource>().tiles.fill(false);
        world.resource_mut::<TileMapResource>().tiles[start_idx] = true;
        let entity = world
            .spawn((
                UnitId(0),
                Position { x: 5.5, y: 5.5 },
                Path {
                    waypoints: Vec::new(),
                },
            ))
            .id();
        world
            .resource_mut::<Messages<TargetReached>>()
            .write(TargetReached { entity });

        let mut schedule = Schedule::default();
        schedule.add_systems(find_path_action);
        schedule.run(&mut world);

        assert!(
            world
                .entity(entity)
                .get::<Path>()
                .unwrap()
                .waypoints
                .is_empty()
        );
    }

    #[test]
    fn movement_stops_before_blocked_waypoint() {
        let mut world = make_world(25, 19);
        let mut tile_map = world.resource_mut::<TileMapResource>();
        tile_map.tiles.fill(true);
        tile_map.tiles[25 + 2] = false;
        let entity = world
            .spawn((
                UnitId(0),
                Position { x: 1.5, y: 1.5 },
                Path {
                    waypoints: vec![(1.5, 1.5), (2.5, 1.5)],
                },
                Speed(DEFAULT_SPEED),
            ))
            .id();
        world.insert_resource(DeltaTime(1.0));

        let mut schedule = Schedule::default();
        schedule.add_systems(move_along_path);
        schedule.run(&mut world);

        let position = world.entity(entity).get::<Position>().unwrap();
        assert_eq!((position.x, position.y), (1.5, 1.5));
        assert!(
            world
                .entity(entity)
                .get::<Path>()
                .unwrap()
                .waypoints
                .is_empty()
        );
    }

    #[test]
    fn different_buildings_have_different_max_progress() {
        let mut world = make_world(25, 19);

        world
            .resource_mut::<Messages<BuildRequest>>()
            .write(BuildRequest {
                col: 1,
                row: 1,
                kind: ObjectKind::Wall,
            });
        world
            .resource_mut::<Messages<BuildRequest>>()
            .write(BuildRequest {
                col: 2,
                row: 1,
                kind: ObjectKind::Bed,
            });
        world
            .resource_mut::<Messages<BuildRequest>>()
            .write(BuildRequest {
                col: 3,
                row: 1,
                kind: ObjectKind::BerryBush,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(construction_system);
        schedule.run(&mut world);

        let queue = world.resource::<ConstructionQueue>();
        assert_eq!(queue.jobs.len(), 3);
        assert_eq!(queue.jobs[0].max_progress, 50.0, "Wall max_progress");
        assert_eq!(queue.jobs[1].max_progress, 80.0, "Bed max_progress");
        assert_eq!(queue.jobs[2].max_progress, 40.0, "BerryBush max_progress");
    }

    #[test]
    fn max_workers_per_job_is_enforced() {
        let mut world = make_world(25, 19);

        let colonist_a = world.spawn((UnitId(0), Position { x: 5.5, y: 5.5 })).id();
        let colonist_b = world.spawn((UnitId(1), Position { x: 5.5, y: 5.5 })).id();
        let _colonist_c = world.spawn((UnitId(2), Position { x: 5.5, y: 5.5 })).id();

        world
            .resource_mut::<ConstructionQueue>()
            .jobs
            .push(ConstructionJob {
                col: 5,
                row: 5,
                kind: ObjectKind::Wall,
                progress: 0.0,
                max_progress: 50.0,
                assigned_units: vec![colonist_a, colonist_b],
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(job_assignment_system);
        schedule.run(&mut world);

        let queue = world.resource::<ConstructionQueue>();
        assert_eq!(
            queue.jobs[0].assigned_units.len(),
            2,
            "third colonist should not be assigned to a full job"
        );
    }
}
