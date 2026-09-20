use data::math::FixedVec2;
use world::{Map, TilePosition};

pub struct Waypoints(pub Vec<FixedVec2>);

impl Waypoints {
    pub fn compute_from_path(start: FixedVec2, end: FixedVec2, path: &[TilePosition]) -> Self {
        let mut out: Vec<FixedVec2> = Vec::new();
        if path.len() <= 1 {
            out.push(end);
            return Self(out);
        }
        for &(tx, ty) in path.iter().skip(1) {
            out.push(Map::tile_centre_sim(tx as u16, ty as u16));
        }
        if let Some(last) = out.last_mut() {
            *last = end;
        }
        let _ = start;
        Self(out)
    }
}
