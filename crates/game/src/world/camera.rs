#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub tile_size: f32,
    pub viewport: [f32; 2],
}

impl Camera {
    pub fn new(tile_size: f32, viewport: [f32; 2]) -> Self {
        Self {
            tile_size,
            viewport,
        }
    }

    pub fn set_viewport(&mut self, viewport: [f32; 2]) {
        self.viewport = viewport;
    }

    pub fn view_projection(&self) -> [[f32; 4]; 4] {
        let w = self.viewport[0].max(1.0);
        let h = self.viewport[1].max(1.0);
        let sx = 2.0 * self.tile_size / w;
        let sy = -2.0 * self.tile_size / h; // flip y: clip space is y-up.

        [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ]
    }
}
