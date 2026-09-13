use core::time::Duration;
use glam::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct Speed(pub f32);

/// A movement order. `waypoints[0]` is the next point to reach; when reached, it is
/// popped. Order is removed when the list is empty.
#[derive(Clone, Debug)]
pub struct UnitOrder {
    pub waypoints: Vec<Vec2>,
}

impl UnitOrder {
    pub fn to(target: Vec2) -> Self {
        Self {
            waypoints: vec![target],
        }
    }

    pub fn path(waypoints: Vec<Vec2>) -> Self {
        Self { waypoints }
    }

    pub fn final_target(&self) -> Option<Vec2> {
        self.waypoints.last().copied()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MoveMarker {
    pub unit: hecs::Entity,
    pub to: Vec2,
    pub remaining: Duration,
}
