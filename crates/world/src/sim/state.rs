use crate::constants::{DEFAULT_MAP_HEIGHT, DEFAULT_MAP_WIDTH};
use crate::sim::camera::Camera;
use crate::sim::map::{Map, Tile};
use crate::sim::selection::{Marquee, Selection};

pub struct World {
    pub map: Map,
    pub camera: Camera,
    pub selection: Selection,
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
        }
    }

    pub fn placeholder(viewport: [f32; 2]) -> Self {
        let width = DEFAULT_MAP_WIDTH;
        let height = DEFAULT_MAP_HEIGHT;
        let tiles = (0..(width as usize * height as usize))
            .map(|i| Tile::Grass {
                variant: (i % 3) as u8,
            })
            .collect();
        let map = Map {
            width,
            height,
            tiles,
        };
        Self::new(map, viewport)
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

    /// Begin or update the drag-selection marquee.
    pub fn set_marquee(&mut self, origin: [f32; 2], current: [f32; 2]) {
        self.selection.marquee = Some(Marquee { origin, current });
    }

    /// Commit the current marquee: run hit-testing and populate `selected`.
    /// Currently a stub; no world entities exist yet.
    pub fn commit_marquee(&mut self) {
        // TODO: hit-test entities within the marquee bounds once entities exist.
        self.selection.marquee = None;
    }

    pub fn clear_marquee(&mut self) {
        self.selection.marquee = None;
    }
}
