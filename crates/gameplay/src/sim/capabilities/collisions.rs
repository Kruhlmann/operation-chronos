use data::{
    geometry::{Footprint, Position},
    math::Scalar,
};

use crate::Sim;

pub trait UnitCollisionHandler {
    fn tick_collision(&mut self);
}

impl UnitCollisionHandler for Sim {
    fn tick_collision(&mut self) {
        let half = Scalar::const_from_int(1).strict_div_int(2);
        let mut units: Vec<(hecs::Entity, Position, Footprint)> = self
            .ecs
            .query::<(&Position, &Footprint)>()
            .iter()
            .map(|(e, (p, f))| (e, *p, *f))
            .collect();
        units.sort_by_key(|u| u.0);

        for _ in 0..3 {
            let mut moved = false;
            for i in 0..units.len() {
                for j in (i + 1)..units.len() {
                    let a = units[i].2.disc_at(units[i].1);
                    let b = units[j].2.disc_at(units[j].1);
                    let Some(sep) = a.separation(&b) else {
                        continue;
                    };
                    let push = sep * half;
                    let new_a = units[i].1 - push;
                    let new_b = units[j].1 + push;
                    if self.map.is_area_passable(units[i].2.disc_at(new_a)) {
                        units[i].1 = new_a;
                        moved = true;
                    }
                    if self.map.is_area_passable(units[j].2.disc_at(new_b)) {
                        units[j].1 = new_b;
                        moved = true;
                    }
                }
            }
            if !moved {
                break;
            }
        }

        for (e, pos, _) in units {
            if let Ok(mut p) = self.ecs.get::<&mut Position>(e) {
                *p = pos;
            }
        }
    }
}
