use crate::constants::{DEFAULT_MAP_HEIGHT, DEFAULT_MAP_WIDTH};
use crate::world::camera::Camera;
use crate::world::map::{Map, Tile};

pub struct World {
    pub map: Map,
    pub camera: Camera,
}

impl World {
    pub fn new(map: Map, viewport: [f32; 2]) -> Self {
        let bounds = map.world_bounds();
        let mut camera = Camera::new(viewport);
        camera.set_bounds(bounds);
        camera.set_center(bounds.center());
        Self { map, camera }
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
}
