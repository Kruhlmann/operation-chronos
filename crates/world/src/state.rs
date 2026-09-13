use core::time::Duration;
use glam::Vec2;

use crate::camera::Camera;
use crate::constants::{
    DEFAULT_MAP_HEIGHT, DEFAULT_MAP_WIDTH, ISO_TILE_HALF_HEIGHT, ORDER_MARKER_RENDER_DURATION,
};
use crate::entity::UnitKind;
use crate::geometry::{Facing, Footprint, Position};
use crate::map::{Map, Tile};
use crate::order::{MoveMarker, Speed, UnitOrder};
use crate::pathfinding;
use crate::selection::{Marquee, Selected, Selection, aabb_contains, point_hits};

const UNIT_PICK_RADIUS: f32 = 32.0;
const TANK_SPEED: f32 = 120.0;
/// Collision/footprint radius in world pixels.
const TANK_FOOTPRINT: f32 = 14.0;

pub struct World {
    pub map: Map,
    pub camera: Camera,
    pub selection: Selection,
    pub ecs: hecs::World,
}

impl World {
    pub fn new(map: Map, viewport: [f32; 2]) -> Self {
        let bounds = map.world_bounds();
        let mut camera = Camera::new(viewport);
        camera.set_bounds(bounds);
        camera.set_center(bounds.center());
        Self {
            map,
            camera,
            selection: Selection::default(),
            ecs: hecs::World::new(),
        }
    }

    pub fn placeholder(viewport: [f32; 2]) -> Self {
        let width = DEFAULT_MAP_WIDTH;
        let height = DEFAULT_MAP_HEIGHT;
        let tiles = (0..(width as usize * height as usize))
            .map(|i| {
                let h = (i as u32).wrapping_mul(2_654_435_761);
                if h.is_multiple_of(12) {
                    Tile::Rock
                } else {
                    Tile::Grass {
                        variant: (i % 3) as u8,
                    }
                }
            })
            .collect();
        let map = Map {
            width,
            height,
            tiles,
        };
        let mut world = Self::new(map, viewport);
        world.spawn_placeholder_tanks();
        world
    }

    fn spawn_placeholder_tanks(&mut self) {
        let center = self.map.world_bounds().center();
        let base = Vec2::new(center[0], center[1]);
        let spacing = 96.0;
        for i in -1..=1_i32 {
            let pos = base + Vec2::new(i as f32 * spacing, 0.0);
            self.clear_footprint_area(pos, TANK_FOOTPRINT);
            self.spawn_tank(pos);
        }
    }

    /// Convert any impassable tiles overlapping the given circular footprint to grass.
    fn clear_footprint_area(&mut self, center: Vec2, radius: f32) {
        let probes = [
            center,
            center + Vec2::new(-radius, 0.0),
            center + Vec2::new(radius, 0.0),
            center + Vec2::new(0.0, -radius),
            center + Vec2::new(0.0, radius),
        ];
        for c in probes {
            let (tx, ty) = Map::get_world_tile_at(c);
            if tx < 0 || ty < 0 || tx >= self.map.width as i32 || ty >= self.map.height as i32 {
                continue;
            }
            let idx = ty as usize * self.map.width as usize + tx as usize;
            if let Some(t) = self.map.tiles.get_mut(idx)
                && !t.is_passable()
            {
                *t = Tile::Grass { variant: 0 };
            }
        }
    }

    pub fn spawn_tank(&mut self, pos: Vec2) -> hecs::Entity {
        self.ecs.spawn((
            Position(pos),
            Facing(0.0),
            UnitKind::Tank,
            Speed(TANK_SPEED),
            Footprint(TANK_FOOTPRINT),
        ))
    }

    pub fn tick(&mut self, dt: Duration) {
        self.run_movement(dt);
        self.resolve_collisions();
        self.tick_markers(dt);
    }

    fn resolve_collisions(&mut self) {
        let mut units: Vec<(hecs::Entity, Vec2, f32)> = self
            .ecs
            .query::<(&Position, &Footprint)>()
            .iter()
            .map(|(e, (p, f))| (e, p.0, f.0))
            .collect();

        for _ in 0..3 {
            let mut moved = false;
            for i in 0..units.len() {
                for j in (i + 1)..units.len() {
                    let (a_pos, a_r) = (units[i].1, units[i].2);
                    let (b_pos, b_r) = (units[j].1, units[j].2);
                    let d = b_pos - a_pos;
                    let dist_sq = d.length_squared();
                    let min_dist = a_r + b_r;
                    if dist_sq >= min_dist * min_dist {
                        continue;
                    }
                    let dist = dist_sq.sqrt();
                    let (normal, overlap) = if dist > 1e-4 {
                        (d / dist, min_dist - dist)
                    } else {
                        (Vec2::X, min_dist)
                    };
                    let push = normal * (overlap * 0.5);
                    let new_a = units[i].1 - push;
                    let new_b = units[j].1 + push;
                    if self.map.is_area_passable(new_a, a_r) {
                        units[i].1 = new_a;
                        moved = true;
                    }
                    if self.map.is_area_passable(new_b, b_r) {
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
                p.0 = pos;
            }
        }
    }

    fn tick_markers(&mut self, dt: Duration) {
        let mut expired: Vec<hecs::Entity> = Vec::new();
        for (e, m) in self.ecs.query_mut::<&mut MoveMarker>() {
            m.remaining = m.remaining.saturating_sub(dt);
            if m.remaining.is_zero() {
                expired.push(e);
            }
        }
        for e in expired {
            let _ = self.ecs.despawn(e);
        }
    }

    fn run_movement(&mut self, dt: Duration) {
        let dt_s = dt.as_secs_f32();
        let mut done: Vec<hecs::Entity> = Vec::new();
        for (e, (pos, facing, speed, footprint, order)) in self.ecs.query_mut::<(
            &mut Position,
            &mut Facing,
            &Speed,
            &Footprint,
            &mut UnitOrder,
        )>() {
            let side = footprint.0;
            let mut remaining_step = speed.0 * dt_s;
            loop {
                let Some(&target) = order.waypoints.first() else {
                    done.push(e);
                    break;
                };
                let to = target - pos.0;
                let dist = to.length();
                if dist <= 1e-3 {
                    order.waypoints.remove(0);
                    continue;
                }
                facing.0 = to.y.atan2(to.x);
                let step = remaining_step.min(dist);
                let delta = to / dist * step;
                let full = pos.0 + delta;
                if self.map.is_area_passable(full, side) {
                    pos.0 = full;
                } else {
                    done.push(e);
                    break;
                }
                if step >= dist {
                    order.waypoints.remove(0);
                    remaining_step -= step;
                    if remaining_step <= 1e-3 {
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
            let _ = self.ecs.remove_one::<UnitOrder>(e);
        }
    }

    pub fn resize(&mut self, viewport: [f32; 2]) {
        self.camera.set_viewport(viewport);
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.camera.pan_by_screen_delta(dx, dy);
    }

    pub fn zoom_at(&mut self, factor: f32, cursor: [f32; 2]) {
        self.camera.zoom_at_cursor(factor, cursor);
    }

    pub fn set_marquee(&mut self, origin: [f32; 2], current: [f32; 2]) {
        self.selection.marquee = Some(Marquee { origin, current });
    }

    pub fn clear_marquee(&mut self) {
        self.selection.marquee = None;
    }

    pub fn commit_marquee(&mut self) {
        let Some(m) = self.selection.marquee.take() else {
            return;
        };
        let a = self.camera.screen_to_world(m.min());
        let b = self.camera.screen_to_world(m.max());
        let min = Vec2::new(a[0].min(b[0]), a[1].min(b[1]));
        let max = Vec2::new(a[0].max(b[0]), a[1].max(b[1]));

        let mut hits: Vec<hecs::Entity> = Vec::new();
        for (e, pos) in self.ecs.query::<&Position>().iter() {
            if aabb_contains(min, max, pos) {
                hits.push(e);
            }
        }
        self.replace_selection(&hits);
    }

    pub fn click_select(&mut self, screen: [f32; 2]) {
        let world_point: Vec2 = self.camera.screen_to_world(screen).into();
        let mut best: Option<(hecs::Entity, f32)> = None;
        for (e, pos) in self.ecs.query::<&Position>().iter() {
            if !point_hits(pos, world_point, UNIT_PICK_RADIUS) {
                continue;
            }
            let d2 = pos.0.distance_squared(world_point);
            if best.map(|(_, b)| d2 < b).unwrap_or(true) {
                best = Some((e, d2));
            }
        }
        let hits: Vec<hecs::Entity> = best.into_iter().map(|(e, _)| e).collect();
        self.replace_selection(&hits);
    }

    pub fn click_order(&mut self, screen: [f32; 2]) {
        let target: Vec2 = self.camera.screen_to_world(screen).into();
        let selected: Vec<hecs::Entity> = self
            .ecs
            .query::<&Selected>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        let existing: Vec<hecs::Entity> = self
            .ecs
            .query::<&MoveMarker>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for e in existing {
            let _ = self.ecs.despawn(e);
        }

        let nearest_goal_tile = Map::get_world_tile_at(target);
        let goal = match pathfinding::nearest_passable(&self.map, nearest_goal_tile, 8) {
            Some(g) => g,
            None => return,
        };

        for e in selected {
            let start_pos = match self.ecs.get::<&Position>(e) {
                Ok(p) => p.0,
                Err(_) => continue,
            };
            let start_tile = Map::get_world_tile_at(start_pos);
            let Some(tile_path) = pathfinding::find_path(&self.map, start_tile, goal) else {
                continue;
            };
            let waypoints = build_waypoints(start_pos, &tile_path, target);
            let final_target = *waypoints.last().unwrap_or(&target);
            let _ = self.ecs.insert_one(e, UnitOrder::path(waypoints));
            self.ecs.spawn((MoveMarker {
                unit: e,
                to: final_target,
                remaining: ORDER_MARKER_RENDER_DURATION,
            },));
        }
    }

    fn replace_selection(&mut self, keep: &[hecs::Entity]) {
        let previously: Vec<hecs::Entity> = self
            .ecs
            .query::<&Selected>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for e in previously {
            let _ = self.ecs.remove_one::<Selected>(e);
        }
        for &e in keep {
            let _ = self.ecs.insert_one(e, Selected);
        }
    }
}

fn build_waypoints(start_pos: Vec2, tile_path: &[(i32, i32)], precise_goal: Vec2) -> Vec<Vec2> {
    let mut out: Vec<Vec2> = Vec::new();
    if tile_path.len() <= 1 {
        out.push(precise_goal);
        return out;
    }
    for &(tx, ty) in tile_path.iter().skip(1) {
        let [ax, ay] = Map::tile_to_world(tx as u16, ty as u16);
        out.push(Vec2::new(ax, ay + ISO_TILE_HALF_HEIGHT));
    }
    if let Some(last) = out.last_mut() {
        *last = precise_goal;
    }
    let _ = start_pos;
    out
}
