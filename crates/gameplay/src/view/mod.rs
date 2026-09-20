pub mod camera;
pub mod selection;

pub use camera::*;
pub use selection::*;

use data::geometry::{Disc, Position};
use data::math::FixedVec2;
use glam::Vec2;

use crate::sim::{PlayerCommand, Sim, UNIT_PICK_RADIUS};

pub struct ClientView {
    pub camera: Camera,
    pub selection: Selection,
}

impl ClientView {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            selection: Selection::default(),
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
        self.selection.marquee = Some(SelectionMarquee { origin, current });
    }

    pub fn clear_marquee(&mut self) {
        self.selection.marquee = None;
    }

    pub fn commit_marquee(&mut self, sim: &Sim) {
        let Some(m) = self.selection.marquee.take() else {
            return;
        };
        let a = self.camera.screen_to_world(m.min());
        let b = self.camera.screen_to_world(m.max());
        let min = FixedVec2::from_render(Vec2::new(a[0].min(b[0]), a[1].min(b[1])));
        let max = FixedVec2::from_render(Vec2::new(a[0].max(b[0]), a[1].max(b[1])));

        let mut hits: Vec<hecs::Entity> = Vec::new();
        for (e, pos) in sim.ecs.query::<&Position>().iter() {
            if pos.is_in_bounds(min, max) {
                hits.push(e);
            }
        }
        self.selection.replace(hits);
    }

    pub fn click_select(&mut self, sim: &Sim, screen: [f32; 2]) {
        let world_render = self.camera.screen_to_world(screen);
        let world_point = Position::from_render(Vec2::new(world_render[0], world_render[1]));
        let mut best: Option<(hecs::Entity, data::math::Scalar)> = None;
        for (e, pos) in sim.ecs.query::<&Position>().iter() {
            if !pos.hits(world_point, UNIT_PICK_RADIUS) {
                continue;
            }
            let d = Disc::new(*pos, UNIT_PICK_RADIUS).distance_to(world_point);
            let d2 = d * d;
            if best.map(|(_, b)| d2 < b).unwrap_or(true) {
                best = Some((e, d2));
            }
        }
        self.selection
            .replace(best.into_iter().map(|(e, _)| e).collect());
    }

    pub fn click_order(&mut self, sim: &mut Sim, screen: [f32; 2]) {
        if self.selection.selected.is_empty() {
            return;
        }
        let world = self.camera.screen_to_world(screen);
        let target = FixedVec2::from_render(Vec2::new(world[0], world[1]));
        sim.schedule_player_command(PlayerCommand::Move {
            units: self.selection.selected.clone(),
            target,
        });
    }
}
