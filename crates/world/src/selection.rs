use glam::Vec2;

use crate::geometry::Position;

pub type EntityId = u64;

#[derive(Clone, Copy, Debug)]
pub struct Marquee {
    pub origin: [f32; 2],
    pub current: [f32; 2],
}

impl Marquee {
    pub fn min(&self) -> [f32; 2] {
        [
            self.origin[0].min(self.current[0]),
            self.origin[1].min(self.current[1]),
        ]
    }

    pub fn max(&self) -> [f32; 2] {
        [
            self.origin[0].max(self.current[0]),
            self.origin[1].max(self.current[1]),
        ]
    }
}

#[derive(Default)]
pub struct Selection {
    pub marquee: Option<Marquee>,
}

#[derive(Clone, Copy, Debug)]
pub struct Selected;

pub fn point_hits(pos: &Position, world_point: Vec2, radius: f32) -> bool {
    pos.0.distance_squared(world_point) <= radius * radius
}

pub fn aabb_contains(min: Vec2, max: Vec2, pos: &Position) -> bool {
    pos.0.x >= min.x && pos.0.x <= max.x && pos.0.y >= min.y && pos.0.y <= max.y
}
