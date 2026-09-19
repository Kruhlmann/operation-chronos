use glam::Vec2;

use crate::{Map, TilePosition, constants::ISOMETRIC_TILE_HALF_HEIGHT};

pub struct Waypoints(pub Vec<Vec2>);

impl Waypoints {
    pub fn compute_from_path(start: Vec2, end: Vec2, path: &[TilePosition]) -> Self {
        let mut out: Vec<Vec2> = Vec::new();
        if path.len() <= 1 {
            out.push(end);
            return Self(out);
        }
        for &(tx, ty) in path.iter().skip(1) {
            let [ax, ay] = Map::tile_to_world(tx as u16, ty as u16);
            out.push(Vec2::new(ax, ay + ISOMETRIC_TILE_HALF_HEIGHT));
        }
        if let Some(last) = out.last_mut() {
            *last = end;
        }
        let _ = start;
        Self(out)
    }
}
