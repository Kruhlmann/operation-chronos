use core::time::Duration;

use data::{
    constants::{DEFAULT_MAP_HEIGHT, DEFAULT_MAP_WIDTH},
    geometry::{Disc, Facing, Footprint, Position},
};
use glam::Vec2;
use world::{
    Map, Tile,
    entity::{Health, MoveMarker, Name, Speed, UnitKind, UnitOrder},
};

const NEAREST_PASSABLE_TILE_SEARCH_RADIUS: i32 = 8;
const UNIT_PICK_RADIUS: f32 = 32.0;
const TANK_SPEED: f32 = 120.0;
const TANK_FOOTPRINT: f32 = 14.0;
const TANK_FOOTPRINT_OFFSET: Vec2 = Vec2::new(0.0, 0.0);
const TANK_MAX_HEALTH: f32 = 100.0;

use crate::{
    camera::Camera,
    pathfinding::{AStarPathFindingAlgorithm, PathFinder, PathFindingResult, Waypoints},
    selection::{Marquee, Selected, Selection},
};

pub struct Simulator {
    pub map: Map,
    pub camera: Camera,
    pub selection: Selection,
    pub ecs: hecs::World,
}

impl Simulator {
    pub fn new(map: Map, viewport: [f32; 2]) -> Self {
        let bounds = map.world_bounds();
        let mut camera = Camera::new(viewport);
        camera.set_bounds(bounds);
        camera.set_center(bounds.get_center());
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
        let map = Map::new(width, height, tiles).unwrap();
        let mut world = Self::new(map, viewport);
        world.spawn_placeholder_tanks();
        world
    }

    pub fn spawn_placeholder_tanks(&mut self) {
        let center = self.map.world_bounds().get_center();
        let base = Vec2::new(center[0], center[1]);
        let spacing = 96.0;
        for i in -1..=1_i32 {
            let pos = Position(base + Vec2::new(i as f32 * spacing, 0.0));
            self.clear_footprint_area(Self::tank_footprint().disc_at(pos));
            self.spawn_tank(pos);
        }
    }

    fn tank_footprint() -> Footprint {
        Footprint::with_offset(TANK_FOOTPRINT, TANK_FOOTPRINT_OFFSET)
    }

    /// Convert any impassable tiles overlapping the given footprint disc to grass.
    fn clear_footprint_area(&mut self, disc: Disc) {
        let probes = disc.axis_probes();
        for c in probes {
            let (tx, ty) = Map::get_world_tile_at(c.0);
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

    pub fn spawn_tank(&mut self, pos: Position) -> hecs::Entity {
        self.ecs.spawn((
            pos,
            Facing(0.0),
            UnitKind::Tank,
            Speed(TANK_SPEED),
            Self::tank_footprint(),
            Name("TANK"),
            Health::new(TANK_MAX_HEALTH),
        ))
    }

    pub fn tick(&mut self, dt: Duration) {
        self.run_movement(dt);
        self.resolve_collisions();
        self.tick_markers(dt);
    }

    fn resolve_collisions(&mut self) {
        let mut units: Vec<(hecs::Entity, Position, Footprint)> = self
            .ecs
            .query::<(&Position, &Footprint)>()
            .iter()
            .map(|(e, (p, f))| (e, *p, *f))
            .collect();

        for _ in 0..3 {
            let mut moved = false;
            for i in 0..units.len() {
                for j in (i + 1)..units.len() {
                    let a = units[i].2.disc_at(units[i].1);
                    let b = units[j].2.disc_at(units[j].1);
                    let Some(sep) = a.separation(&b) else {
                        continue;
                    };
                    let push = sep * 0.5;
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
                if self.map.is_area_passable(footprint.disc_at(Position(full))) {
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
            if pos.is_in_bounds(min, max) {
                hits.push(e);
            }
        }
        self.replace_selection(&hits);
    }

    pub fn click_select(&mut self, screen: [f32; 2]) {
        let world_point = Position(self.camera.screen_to_world(screen).into());
        let mut best: Option<(hecs::Entity, f32)> = None;
        for (e, pos) in self.ecs.query::<&Position>().iter() {
            if !pos.hits(world_point, UNIT_PICK_RADIUS) {
                continue;
            }
            let d2 = Disc::new(*pos, UNIT_PICK_RADIUS)
                .distance_to(world_point)
                .powi(2);
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
        let goal = match self.map.find_nearest_passable_tile_in_radius(
            nearest_goal_tile,
            NEAREST_PASSABLE_TILE_SEARCH_RADIUS,
        ) {
            Some(g) => g,
            None => return,
        };

        for unit in selected {
            let start_pos = match self.ecs.get::<&Position>(unit) {
                Ok(p) => p.0,
                Err(_) => continue,
            };
            let start_tile = Map::get_world_tile_at(start_pos);
            let PathFindingResult::Path(tile_path) =
                PathFinder::find_path::<AStarPathFindingAlgorithm>(&self.map, start_tile, goal)
            else {
                continue;
            };
            let Waypoints(waypoints) = Waypoints::compute_from_path(start_pos, target, &tile_path);
            let final_target = *waypoints.last().unwrap_or(&target);
            let _ = self.ecs.insert_one(unit, UnitOrder::path(waypoints));
            self.ecs.spawn((MoveMarker::new(unit, final_target),));
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

    pub fn temp_create_map() -> Map {
        let mut tiles: Vec<Tile> = Vec::new();
        for y in 0u32..64 {
            for x in 0u32..64 {
                if x.is_multiple_of(5) && y.is_multiple_of(3) {
                    tiles.push(Tile::Rock);
                } else if x.is_multiple_of(2) && y.is_multiple_of(9) {
                    tiles.push(Tile::Grass { variant: 1 });
                } else {
                    tiles.push(Tile::Grass { variant: 0 });
                }
            }
        }
        Map::new(64, 64, tiles).unwrap()
    }
}
