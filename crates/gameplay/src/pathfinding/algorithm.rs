use std::{
    cmp::Ordering,
    collections::{BTreeMap, BinaryHeap},
};

use world::map::{Map, TilePosition};

pub struct OctileDistance(pub i32);

impl OctileDistance {
    pub const CONST_UNIT: i32 = 1000;
    /// `~sqrt(2) * UNIT`.
    pub const COST_DIAGONAL_UNIT: i32 = 1414;

    pub fn calculate(a: TilePosition, b: TilePosition) -> Self {
        let dx = (a.0 - b.0).abs();
        let dy = (a.1 - b.1).abs();
        let (min, max) = if dx < dy { (dx, dy) } else { (dy, dx) };
        Self(Self::CONST_UNIT * (max - min) + Self::COST_DIAGONAL_UNIT * min)
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
        other
            .f
            .cmp(&self.f)
            .then_with(|| other.node.cmp(&self.node))
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
        came_from: &BTreeMap<TilePosition, TilePosition>,
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

    #[inline]
    fn step_cost(from: TilePosition, to: TilePosition) -> i32 {
        let dx = (from.0 - to.0).abs();
        let dy = (from.1 - to.1).abs();
        if dx + dy == 2 {
            OctileDistance::COST_DIAGONAL_UNIT
        } else {
            OctileDistance::CONST_UNIT
        }
    }
}

impl PathFindingAlgorithm for AStarPathFindingAlgorithm {
    fn compute_path(
        map: &Map,
        start: TilePosition,
        goal: TilePosition,
    ) -> Option<Vec<TilePosition>> {
        let mut open: BinaryHeap<HeapEntry> = BinaryHeap::new();
        let mut came_from: BTreeMap<TilePosition, TilePosition> = BTreeMap::new();
        let mut g_score: BTreeMap<TilePosition, i32> = BTreeMap::new();
        g_score.insert(start, 0);
        let OctileDistance(f) = OctileDistance::calculate(start, goal);
        open.push(HeapEntry { f, node: start });

        while let Some(HeapEntry { node: current, .. }) = open.pop() {
            if current == goal {
                let found_path = Self::reconstruct_path(&came_from, current);
                return Some(found_path);
            }
            let g_current = *g_score.get(&current).unwrap_or(&i32::MAX);
            for neighbor in map.get_tile_neighbors_unchecked_octile(current) {
                if !map.is_passable(neighbor.0, neighbor.1) {
                    continue;
                }
                // Forbid corner-cutting: a diagonal step is only legal when
                // both orthogonal neighbours it slips between are also
                // passable.
                let dx = neighbor.0 - current.0;
                let dy = neighbor.1 - current.1;
                if dx != 0
                    && dy != 0
                    && (!map.is_passable(current.0 + dx, current.1)
                        || !map.is_passable(current.0, current.1 + dy))
                {
                    continue;
                }
                let tentative = g_current + Self::step_cost(current, neighbor);
                let prev = *g_score.get(&neighbor).unwrap_or(&i32::MAX);
                if tentative < prev {
                    came_from.insert(neighbor, current);
                    g_score.insert(neighbor, tentative);
                    let OctileDistance(h) = OctileDistance::calculate(neighbor, goal);
                    open.push(HeapEntry {
                        f: tentative + h,
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
        // Octile A* can slip diagonally past the single blocker; two diagonal
        // moves + two straight moves = 5 nodes total.
        assert_eq!(p.len(), 5);
    }

    #[test]
    fn no_path_when_blocked() {
        let m = map_from(&["...", "###", "..."]);
        assert!(AStarPathFindingAlgorithm::compute_path(&m, (0, 0), (2, 2)).is_none());
    }
}
