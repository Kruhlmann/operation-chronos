use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::map::Map;

type Node = (i32, i32);

pub fn find_path(map: &Map, start: Node, goal: Node) -> Option<Vec<Node>> {
    if !map.is_passable(goal.0, goal.1) {
        return None;
    }
    if start == goal {
        return Some(vec![start]);
    }

    let mut open: BinaryHeap<HeapEntry> = BinaryHeap::new();
    let mut came_from: HashMap<Node, Node> = HashMap::new();
    let mut g_score: HashMap<Node, i32> = HashMap::new();

    g_score.insert(start, 0);
    open.push(HeapEntry {
        f: manhattan(start, goal),
        node: start,
    });

    while let Some(HeapEntry { node: current, .. }) = open.pop() {
        if current == goal {
            return Some(reconstruct(&came_from, current));
        }
        let g_current = *g_score.get(&current).unwrap_or(&i32::MAX);
        for n in neighbors4(current) {
            if !map.is_passable(n.0, n.1) {
                continue;
            }
            let tentative = g_current + 1;
            let prev = *g_score.get(&n).unwrap_or(&i32::MAX);
            if tentative < prev {
                came_from.insert(n, current);
                g_score.insert(n, tentative);
                open.push(HeapEntry {
                    f: tentative + manhattan(n, goal),
                    node: n,
                });
            }
        }
    }

    None
}

pub fn nearest_passable(map: &Map, goal: Node, radius: i32) -> Option<Node> {
    if map.is_passable(goal.0, goal.1) {
        return Some(goal);
    }
    for r in 1..=radius {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs() != r && dy.abs() != r {
                    continue;
                }
                let n = (goal.0 + dx, goal.1 + dy);
                if map.is_passable(n.0, n.1) {
                    return Some(n);
                }
            }
        }
    }
    None
}

fn neighbors4(n: Node) -> [Node; 4] {
    [
        (n.0 + 1, n.1),
        (n.0 - 1, n.1),
        (n.0, n.1 + 1),
        (n.0, n.1 - 1),
    ]
}

fn manhattan(a: Node, b: Node) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

fn reconstruct(came_from: &HashMap<Node, Node>, mut current: Node) -> Vec<Node> {
    let mut path = vec![current];
    while let Some(&prev) = came_from.get(&current) {
        path.push(prev);
        current = prev;
    }
    path.reverse();
    path
}

#[derive(Eq, PartialEq)]
struct HeapEntry {
    f: i32,
    node: Node,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Tile;

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
        Map {
            width: w,
            height: h,
            tiles,
        }
    }

    #[test]
    fn straight_line() {
        let m = map_from(&["....."]);
        let p = find_path(&m, (0, 0), (4, 0)).unwrap();
        assert_eq!(p.len(), 5);
    }

    #[test]
    fn detours_around_wall() {
        let m = map_from(&[".....", "..#..", "....."]);
        let p = find_path(&m, (0, 1), (4, 1)).unwrap();
        // 4 straight + 2 detour = 6 steps → 7 nodes.
        assert_eq!(p.len(), 7);
    }

    #[test]
    fn no_path_when_blocked() {
        let m = map_from(&["...", "###", "..."]);
        assert!(find_path(&m, (0, 0), (2, 2)).is_none());
    }

    #[test]
    fn goal_impassable_returns_none() {
        let m = map_from(&["...", ".#.", "..."]);
        assert!(find_path(&m, (0, 0), (1, 1)).is_none());
    }
}
