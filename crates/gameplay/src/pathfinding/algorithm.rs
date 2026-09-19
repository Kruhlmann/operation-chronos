use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    ops::{Add, Sub},
};

use num_traits::Signed;

use world::map::{Map, TilePosition};

pub struct ManhattenDistance<T>(T);

impl<T: Add + Sub + Signed> ManhattenDistance<T> {
    pub fn calculate(a: (T, T), b: (T, T)) -> ManhattenDistance<T> {
        ManhattenDistance((a.0 - b.0).abs() + (a.1 - b.1).abs())
    }
}

pub trait PathFindingAlgorithm {
    fn compute_path(
        map: &Map,
        start: TilePosition,
        goal: TilePosition,
    ) -> Option<Vec<TilePosition>>;
}

#[derive(Eq, PartialEq)]
pub struct HeapEntry {
    pub f: i32,
    pub node: TilePosition,
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f)
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct AStarPathFindingAlgorithm;

impl AStarPathFindingAlgorithm {
    fn reconstruct_path(
        came_from: &HashMap<TilePosition, TilePosition>,
        mut current: TilePosition,
    ) -> Vec<TilePosition> {
        let mut path = vec![current];
        while let Some(&prev) = came_from.get(&current) {
            path.push(prev);
            current = prev;
        }
        path.reverse();
        path
    }
}

impl PathFindingAlgorithm for AStarPathFindingAlgorithm {
    fn compute_path(
        map: &Map,
        start: TilePosition,
        goal: TilePosition,
    ) -> Option<Vec<TilePosition>> {
        let mut open: BinaryHeap<HeapEntry> = BinaryHeap::new();
        let mut came_from: HashMap<TilePosition, TilePosition> = HashMap::new();
        let mut g_score: HashMap<TilePosition, i32> = HashMap::new();
        g_score.insert(start, 0);
        let ManhattenDistance(f) = ManhattenDistance::calculate(start, goal);
        open.push(HeapEntry { f, node: start });

        while let Some(HeapEntry { node: current, .. }) = open.pop() {
            if current == goal {
                let found_path = Self::reconstruct_path(&came_from, current);
                return Some(found_path);
            }
            let g_current = *g_score.get(&current).unwrap_or(&i32::MAX);
            for neighbor in map.get_tile_neighbors_unchecked(current) {
                if !map.is_passable(neighbor.0, neighbor.1) {
                    continue;
                }
                let tentative = g_current + 1;
                let prev = *g_score.get(&neighbor).unwrap_or(&i32::MAX);
                if tentative < prev {
                    came_from.insert(neighbor, current);
                    g_score.insert(neighbor, tentative);
                    let ManhattenDistance(f) = ManhattenDistance::calculate(neighbor, goal);
                    open.push(HeapEntry {
                        f: tentative + f,
                        node: neighbor,
                    });
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use world::Tile;

    fn map_from(rows: &[&str]) -> Map {
        let h = rows.len() as u16;
        let w = rows[0].len() as u16;
        let mut tiles = Vec::with_capacity((w * h) as usize);
        for row in rows {
            for c in row.chars() {
                tiles.push(if c == '#' {
                    Tile::Rock
                } else {
                    Tile::Grass { variant: 0 }
                });
            }
        }
        Map::new(w, h, tiles).unwrap()
    }

    #[test]
    fn straight_line() {
        let m = map_from(&["....."]);
        let p = AStarPathFindingAlgorithm::compute_path(&m, (0, 0), (4, 0)).unwrap();
        assert_eq!(p.len(), 5);
    }

    #[test]
    fn detours_around_wall() {
        let m = map_from(&[".....", "..#..", "....."]);
        let p = AStarPathFindingAlgorithm::compute_path(&m, (0, 1), (4, 1)).unwrap();
        // 4 straight + 2 detour = 6 steps → 7 nodes.
        assert_eq!(p.len(), 7);
    }

    #[test]
    fn no_path_when_blocked() {
        let m = map_from(&["...", "###", "..."]);
        assert!(AStarPathFindingAlgorithm::compute_path(&m, (0, 0), (2, 2)).is_none());
    }
}
