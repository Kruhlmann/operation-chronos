use core::time::Duration;

use world::entity::UnitMovementMarker;

pub fn tick_markers(ecs: &mut hecs::World, dt: Duration) {
    let mut expired: Vec<hecs::Entity> = Vec::new();
    for (e, m) in ecs.query_mut::<&mut UnitMovementMarker>() {
        m.remaining = m.remaining.saturating_sub(dt);
        if m.remaining.is_zero() {
            expired.push(e);
        }
    }
    for e in expired {
        let _ = ecs.despawn(e);
    }
}
