use data::constants::TICK_HZ;
use data::geometry::{Facing, Footprint, Position};
use data::math::{FacingVec2, SCALAR_EPSILON, Scalar};
use world::Map;
use world::entity::{Speed, UnitMoveInstructions};

/// Advance all units with an active [`UnitOrder`] by one simulation tick.
pub fn run_movement(ecs: &mut hecs::World, map: &Map) {
    let hz = Scalar::from_num(TICK_HZ);
    let mut done: Vec<hecs::Entity> = Vec::new();
    for (e, (pos, facing, speed, footprint, order)) in ecs.query_mut::<(
        &mut Position,
        &mut Facing,
        &Speed,
        &Footprint,
        &mut UnitMoveInstructions,
    )>() {
        // Convert speed-per-second to speed-per-tick once and reuse.
        let mut remaining_step = speed.0 / hz;
        loop {
            let Some(&target) = order.waypoints.first() else {
                done.push(e);
                break;
            };
            let to = target - pos.0;
            let dist_sq = to.length_squared();
            if dist_sq <= SCALAR_EPSILON {
                order.waypoints.remove(0);
                continue;
            }
            let dist = to.length();
            if let Some(new_facing) = FacingVec2::from_direction(to) {
                facing.0 = new_facing;
            }
            let step = if remaining_step < dist {
                remaining_step
            } else {
                dist
            };
            let delta = to / dist * step;
            let full = Position(pos.0 + delta);
            if map.is_area_passable(footprint.disc_at(full)) {
                *pos = full;
            } else {
                done.push(e);
                break;
            }
            if step >= dist {
                order.waypoints.remove(0);
                remaining_step -= step;
                if remaining_step <= SCALAR_EPSILON {
                    if order.waypoints.is_empty() {
                        done.push(e);
                    }
                    break;
                }
            } else {
                break;
            }
        }
    }
    for e in done {
        let _ = ecs.remove_one::<UnitMoveInstructions>(e);
    }
}
