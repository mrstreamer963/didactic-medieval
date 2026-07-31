use bevy_ecs::prelude::*;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

#[derive(Resource)]
pub struct SimulationRng(pub StdRng);

#[derive(Resource)]
pub struct DeltaTime(pub f32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectKind {
    Wall,
    Bed,
    BerryBush,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MapTileObject {
    Building(ObjectKind),
    ConstructionSite(ObjectKind),
    FoodSource(ObjectKind, u8),
}

#[derive(Resource)]
pub struct MapObjects {
    pub tiles: Vec<Option<MapTileObject>>,
}

impl MapObjects {
    pub fn new(cols: u32, rows: u32) -> Self {
        MapObjects {
            tiles: vec![None; (cols * rows) as usize],
        }
    }
}

#[derive(Resource)]
pub struct TileMapResource {
    pub tiles: Vec<bool>,
    pub cols: u32,
    pub rows: u32,
}

impl TileMapResource {
    pub const COLS: u32 = 25;
    pub const ROWS: u32 = 19;
    pub const BLOCKED_RATIO: f64 = 0.12;
    pub const MAX_SPAWN_ATTEMPTS: u32 = 50;

    pub fn new(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let total = (Self::COLS * Self::ROWS) as usize;
        let blocked_count = (total as f64 * Self::BLOCKED_RATIO).round() as usize;
        let mut tiles = vec![true; total];

        let mut indices: Vec<usize> = (0..total).collect();
        for i in (1..total).rev() {
            let j = rng.gen_range(0..=i);
            indices.swap(i, j);
        }
        for i in 0..blocked_count {
            tiles[indices[i]] = false;
        }

        TileMapResource {
            tiles,
            cols: Self::COLS,
            rows: Self::ROWS,
        }
    }

    pub fn world_to_tile(&self, x: f32, y: f32) -> (u32, u32) {
        let col = x.floor() as u32;
        let row = y.floor() as u32;
        (col.min(self.cols - 1), row.min(self.rows - 1))
    }

    pub fn tile_to_world(&self, col: u32, row: u32) -> (f32, f32) {
        let cx = col as f32 + 0.5;
        let cy = row as f32 + 0.5;
        (cx, cy)
    }

    pub fn is_walkable(&self, col: u32, row: u32) -> bool {
        if col >= self.cols || row >= self.rows {
            return false;
        }
        let idx = (row * self.cols + col) as usize;
        self.tiles[idx]
    }

    pub fn random_walkable_tile(&self, rng: &mut StdRng) -> (u32, u32) {
        for _ in 0..Self::MAX_SPAWN_ATTEMPTS {
            let col = rng.gen_range(0..self.cols);
            let row = rng.gen_range(0..self.rows);
            if self.is_walkable(col, row) {
                return (col, row);
            }
        }

        for row in 0..self.rows {
            for col in 0..self.cols {
                if self.is_walkable(col, row) {
                    return (col, row);
                }
            }
        }

        (0, 0)
    }
}

#[derive(Clone)]
pub struct ConstructionJob {
    pub col: u32,
    pub row: u32,
    pub kind: ObjectKind,
    pub progress: f32,
    pub max_progress: f32,
    pub assigned_units: Vec<Entity>,
}

#[derive(Resource)]
pub struct ConstructionQueue {
    pub jobs: Vec<ConstructionJob>,
}

impl ConstructionQueue {
    pub fn new() -> Self {
        ConstructionQueue { jobs: Vec::new() }
    }
}
