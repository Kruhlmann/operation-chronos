use core::time::Duration;

use world::entity::UnitMovementMarker;

use crate::Sim;

pub trait UnitMovementMarkerHandler {
    fn tick_movement_markers(&mut self, dt: Duration);
}

impl UnitMovementMarkerHandler for Sim {
    fn tick_movement_markers(&mut self, dt: Duration) {
        let mut expired: Vec<hecs::Entity> = Vec::new();
        for (e, m) in self.ecs.query_mut::<&mut UnitMovementMarker>() {
            m.remaining = m.remaining.saturating_sub(dt);
            if m.remaining.is_zero() {
                expired.push(e);
            }
        }
        for e in expired {
            let _ = self.ecs.despawn(e);
        }
    }
}
