use core::time::Duration;
use glam::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct Speed(pub f32);

#[derive(Clone, Copy, Debug)]
pub enum UnitOrder {
    Move(Vec2),
}

#[derive(Clone, Copy, Debug)]
pub struct MoveMarker {
    pub unit: hecs::Entity,
    pub to: Vec2,
    pub remaining: Duration,
}
