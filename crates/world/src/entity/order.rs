use core::time::Duration;

use data::constants::ORDER_MARKER_RENDER_DURATION;
use glam::Vec2;

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
    pub target_position: Vec2,
    pub remaining: Duration,
}

impl MoveMarker {
    pub fn new(unit: hecs::Entity, target_position: Vec2) -> Self {
        Self {
            unit,
            target_position,
            remaining: ORDER_MARKER_RENDER_DURATION,
        }
    }
}
