use crate::{Map, TilePosition, pathfinding::PathFindingAlgorithm};

#[derive(Debug, Eq, PartialEq)]
pub enum PathFindingResult {
    NoLegalPath,
    EmptyPath,
    Path(Vec<TilePosition>),
}

pub struct PathFinder;

impl PathFinder {
    pub fn find_path<Algorithm: PathFindingAlgorithm>(
        map: &Map,
        start: TilePosition,
        goal: TilePosition,
    ) -> PathFindingResult {
        if !map.is_passable(goal.0, goal.1) {
            return PathFindingResult::NoLegalPath;
        }
        if start == goal {
            return PathFindingResult::EmptyPath;
        }
        Algorithm::compute_path(map, start, goal)
            .map(PathFindingResult::Path)
            .unwrap_or(PathFindingResult::EmptyPath)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tile;

    struct AlgorithmFindsSevenSevenPath;
    impl PathFindingAlgorithm for AlgorithmFindsSevenSevenPath {
        fn compute_path(
            _map: &Map,
            _start: TilePosition,
            _goal: TilePosition,
        ) -> Option<Vec<TilePosition>> {
            Some(vec![(7, 7)])
        }
    }

    #[test]
    fn returns_no_legal_path_when_goal_is_impassable() {
        let map = Map::new(
            2,
            2,
            vec![
                Tile::Grass { variant: 0 },
                Tile::Grass { variant: 0 },
                Tile::Grass { variant: 0 },
                Tile::Rock,
            ],
        )
        .unwrap();
        let result = PathFinder::find_path::<AlgorithmFindsSevenSevenPath>(&map, (0, 0), (1, 1));
        assert_eq!(result, PathFindingResult::NoLegalPath);
    }

    #[test]
    fn returns_empty_path_when_start_is_goal() {
        let map = Map::new(1, 1, vec![Tile::Grass { variant: 0 }]).unwrap();
        let result = PathFinder::find_path::<AlgorithmFindsSevenSevenPath>(&map, (0, 0), (0, 0));
        assert_eq!(result, PathFindingResult::EmptyPath);
    }

    #[test]
    fn returns_path_when_arguments_are_legal() {
        let map = Map::new(
            2,
            2,
            vec![
                Tile::Grass { variant: 0 },
                Tile::Grass { variant: 0 },
                Tile::Grass { variant: 0 },
                Tile::Grass { variant: 0 },
            ],
        )
        .unwrap();

        let result = PathFinder::find_path::<AlgorithmFindsSevenSevenPath>(&map, (0, 0), (1, 1));
        assert_eq!(result, PathFindingResult::Path(vec![(7, 7)]));
    }

    struct AlgorithmNeverFindsPath;
    impl PathFindingAlgorithm for AlgorithmNeverFindsPath {
        fn compute_path(
            _map: &Map,
            _start: TilePosition,
            _goal: TilePosition,
        ) -> Option<Vec<TilePosition>> {
            None
        }
    }

    #[test]
    fn returns_empty_path_result_when_algorithm_returns_none() {
        let map = Map::new(
            2,
            2,
            vec![
                Tile::Grass { variant: 0 },
                Tile::Grass { variant: 0 },
                Tile::Grass { variant: 0 },
                Tile::Grass { variant: 0 },
            ],
        )
        .unwrap();

        let result = PathFinder::find_path::<AlgorithmNeverFindsPath>(&map, (0, 0), (1, 1));
        assert_eq!(result, PathFindingResult::EmptyPath);
    }
}
