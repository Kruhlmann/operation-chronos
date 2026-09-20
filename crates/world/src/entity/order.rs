use core::time::Duration;

use data::constants::ORDER_MARKER_RENDER_DURATION;
use data::math::FixedVec2;

#[derive(Clone, Debug)]
pub struct UnitMoveInstructions {
    pub waypoints: Vec<FixedVec2>,
}

impl UnitMoveInstructions {
    pub fn to(target: FixedVec2) -> Self {
        Self {
            waypoints: vec![target],
        }
    }

    pub fn path(waypoints: Vec<FixedVec2>) -> Self {
        Self { waypoints }
    }

    pub fn final_target(&self) -> Option<FixedVec2> {
        self.waypoints.last().copied()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UnitMovementMarker {
    pub unit: hecs::Entity,
    pub target_position: FixedVec2,
    pub remaining: Duration,
}

impl UnitMovementMarker {
    pub fn new(unit: hecs::Entity, target_position: FixedVec2) -> Self {
        Self {
            unit,
            target_position,
            remaining: ORDER_MARKER_RENDER_DURATION,
        }
    }
}
