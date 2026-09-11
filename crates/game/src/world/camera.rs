use crate::constants::{CAMERA_MAX_ZOOM, CAMERA_MIN_ZOOM};
use crate::world::map::WorldBounds;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub center: [f32; 2],
    pub zoom: f32,
    pub viewport: [f32; 2],
    pub bounds: WorldBounds,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Camera {
    pub fn new(viewport: [f32; 2]) -> Self {
        Self {
            center: [0.0, 0.0],
            zoom: 1.0,
            viewport,
            bounds: WorldBounds {
                min: [0.0, 0.0],
                max: [0.0, 0.0],
            },
            min_zoom: CAMERA_MIN_ZOOM,
            max_zoom: CAMERA_MAX_ZOOM,
        }
    }

    pub fn set_viewport(&mut self, viewport: [f32; 2]) {
        self.viewport = viewport;
        self.clamp_center();
    }

    pub fn set_bounds(&mut self, bounds: WorldBounds) {
        self.bounds = bounds;
        self.clamp_center();
    }

    pub fn set_center(&mut self, center: [f32; 2]) {
        self.center = center;
        self.clamp_center();
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(self.min_zoom, self.max_zoom);
        self.clamp_center();
    }

    pub fn pan_by_screen_delta(&mut self, dx: f32, dy: f32) {
        self.center[0] -= dx / self.zoom;
        self.center[1] -= dy / self.zoom;
        self.clamp_center();
    }

    pub fn zoom_at_cursor(&mut self, factor: f32, cursor: [f32; 2]) {
        let old_zoom = self.zoom;
        let new_zoom = (self.zoom * factor).clamp(self.min_zoom, self.max_zoom);
        if new_zoom == old_zoom {
            return;
        }

        let world = self.screen_to_world(cursor);
        self.zoom = new_zoom;
        let world_after = self.screen_to_world(cursor);
        self.center[0] += world[0] - world_after[0];
        self.center[1] += world[1] - world_after[1];
        self.clamp_center();
    }

    pub fn screen_to_world(&self, screen: [f32; 2]) -> [f32; 2] {
        let offset_x = screen[0] - self.viewport[0] * 0.5;
        let offset_y = screen[1] - self.viewport[1] * 0.5;
        [
            self.center[0] + offset_x / self.zoom,
            self.center[1] + offset_y / self.zoom,
        ]
    }

    fn clamp_center(&mut self) {
        for axis in 0..2 {
            let min = self.bounds.min[axis];
            let max = self.bounds.max[axis];
            if max > min {
                self.center[axis] = self.center[axis].clamp(min, max);
            } else {
                self.center[axis] = (min + max) * 0.5;
            }
        }
    }

    pub fn view_projection(&self) -> [[f32; 4]; 4] {
        let w = self.viewport[0].max(1.0);
        let h = self.viewport[1].max(1.0);
        let sx = 2.0 * self.zoom / w;
        let sy = -2.0 * self.zoom / h;
        let tx = -self.center[0] * sx;
        let ty = -self.center[1] * sy;

        [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [tx, ty, 0.0, 1.0],
        ]
    }
}
