use std::collections::BinaryHeap;

use rand::Rng;
use rand::rngs::StdRng;

use crate::resources::TileMapResource;

pub const MAX_PATH_ATTEMPTS: u32 = 20;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Node {
    f_score: u32,
    col: u32,
    row: u32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn heuristic(a: (u32, u32), b: (u32, u32)) -> u32 {
    let dx = a.0.abs_diff(b.0);
    let dy = a.1.abs_diff(b.1);
    dx + dy
}

const DIRS: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

pub fn astar(
    start: (u32, u32),
    goal: (u32, u32),
    tile_map: &TileMapResource,
) -> Option<Vec<(u32, u32)>> {
    if start == goal {
        return Some(vec![start]);
    }

    if !tile_map.is_walkable(start.0, start.1) || !tile_map.is_walkable(goal.0, goal.1) {
        return None;
    }

    let cols = tile_map.cols as usize;
    let rows = tile_map.rows as usize;
    let total = cols * rows;

    let mut g_scores = vec![u32::MAX; total];
    let mut came_from = vec![u32::MAX; total];

    let sidx = start.1 as usize * cols + start.0 as usize;
    g_scores[sidx] = 0;

    let mut heap = BinaryHeap::new();
    heap.push(Node {
        f_score: heuristic(start, goal),
        col: start.0,
        row: start.1,
    });

    while let Some(current) = heap.pop() {
        if (current.col, current.row) == goal {
            let mut path = Vec::new();
            let mut cur = (current.col, current.row);
            loop {
                path.push(cur);
                if cur == start {
                    break;
                }
                let idx = cur.1 as usize * cols + cur.0 as usize;
                let prev = came_from[idx];
                cur = ((prev % cols as u32), (prev / cols as u32));
            }
            path.reverse();
            return Some(path);
        }

        let cg = g_scores[current.row as usize * cols + current.col as usize];
        if cg == u32::MAX {
            continue;
        }

        for &(dc, dr) in &DIRS {
            let nc = current.col as i32 + dc;
            let nr = current.row as i32 + dr;
            if nc < 0 || nr < 0 || nc >= cols as i32 || nr >= rows as i32 {
                continue;
            }
            let nc = nc as u32;
            let nr = nr as u32;
            if !tile_map.is_walkable(nc, nr) {
                continue;
            }

            let ng = cg + 1;
            let nidx = nr as usize * cols + nc as usize;
            if ng < g_scores[nidx] {
                g_scores[nidx] = ng;
                came_from[nidx] = current.row * cols as u32 + current.col;
                let f = ng + heuristic((nc, nr), goal);
                heap.push(Node {
                    f_score: f,
                    col: nc,
                    row: nr,
                });
            }
        }
    }

    None
}

pub fn tile_path_to_waypoints(path: &[(u32, u32)], tile_map: &TileMapResource) -> Vec<(f32, f32)> {
    path.iter()
        .map(|&(col, row)| tile_map.tile_to_world(col, row))
        .collect()
}

pub fn random_reachable_path(
    start: (u32, u32),
    tile_map: &TileMapResource,
    rng: &mut StdRng,
) -> Option<Vec<(u32, u32)>> {
    if !tile_map.is_walkable(start.0, start.1) {
        return None;
    }

    for _ in 0..MAX_PATH_ATTEMPTS {
        let goal = (
            rng.gen_range(0..tile_map.cols),
            rng.gen_range(0..tile_map.rows),
        );
        if let Some(path) = astar(start, goal, tile_map) {
            return Some(path);
        }
    }

    None
}
