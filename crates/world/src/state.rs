use core::time::Duration;
use glam::Vec2;

use crate::camera::Camera;
use crate::constants::{DEFAULT_MAP_HEIGHT, DEFAULT_MAP_WIDTH, ORDER_MARKER_RENDER_DURATION};
use crate::entity::UnitKind;
use crate::geometry::{Facing, Footprint, Position};
use crate::map::{Map, Tile};
use crate::order::{MoveMarker, Speed, UnitOrder};
use crate::selection::{Marquee, Selected, Selection, aabb_contains, point_hits};

const UNIT_PICK_RADIUS: f32 = 32.0;
const TANK_SPEED: f32 = 120.0;
const TANK_FOOTPRINT: f32 = 0.7;

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
                // Deterministic scatter of rocks (~8%) using a cheap hash.
                let h = (i as u32).wrapping_mul(2_654_435_761);
                if h % 12 == 0 {
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

    /// Convert any impassable tiles overlapping the given diamond footprint to grass.
    fn clear_footprint_area(&mut self, center: Vec2, tiles: f32) {
        use crate::constants::{ISO_TILE_HALF_HEIGHT, ISO_TILE_HALF_WIDTH};
        let hw = tiles * ISO_TILE_HALF_WIDTH;
        let hh = tiles * ISO_TILE_HALF_HEIGHT;
        let probes = [
            center,
            center + Vec2::new(-hw, 0.0),
            center + Vec2::new(hw, 0.0),
            center + Vec2::new(0.0, -hh),
            center + Vec2::new(0.0, hh),
        ];
        for c in probes {
            let (tx, ty) = Map::world_to_tile(c);
            if tx < 0 || ty < 0 || tx >= self.map.width as i32 || ty >= self.map.height as i32 {
                continue;
            }
            let idx = ty as usize * self.map.width as usize + tx as usize;
            if let Some(t) = self.map.tiles.get_mut(idx) {
                if !t.passable() {
                    *t = Tile::Grass { variant: 0 };
                }
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
        self.tick_markers(dt);
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
        let mut arrived: Vec<hecs::Entity> = Vec::new();
        for (e, (pos, facing, speed, footprint, order)) in
            self.ecs
                .query_mut::<(&mut Position, &mut Facing, &Speed, &Footprint, &UnitOrder)>()
        {
            let UnitOrder::Move(target) = *order;
            let to = target - pos.0;
            let dist = to.length();
            if dist <= 1e-3 {
                arrived.push(e);
                continue;
            }
            facing.0 = to.y.atan2(to.x);
            let step = (speed.0 * dt_s).min(dist);
            let delta = to / dist * step;
            let side = footprint.0;
            let full = pos.0 + delta;
            if self.map.is_area_passable(full, side) {
                pos.0 = full;
            } else {
                let slide_x = pos.0 + Vec2::new(delta.x, 0.0);
                let slide_y = pos.0 + Vec2::new(0.0, delta.y);
                let ok_x = delta.x != 0.0 && self.map.is_area_passable(slide_x, side);
                let ok_y = delta.y != 0.0 && self.map.is_area_passable(slide_y, side);
                if ok_x {
                    pos.0 = slide_x;
                } else if ok_y {
                    pos.0 = slide_y;
                } else {
                    arrived.push(e);
                    continue;
                }
            }
            if step >= dist {
                arrived.push(e);
            }
        }
        for e in arrived {
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

        for e in selected {
            let _ = self.ecs.insert_one(e, UnitOrder::Move(target));
            self.ecs.spawn((MoveMarker {
                unit: e,
                to: target,
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
