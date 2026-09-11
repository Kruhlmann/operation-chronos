/// 2D camera mapping world-pixel space to clip space.
///
/// The world is expressed in pixels (tiles are pre-projected to pixel
/// positions by the map). The camera applies pan (`center`) and `zoom`, then
/// an orthographic projection sized to the viewport. Rotation could be added
/// later purely inside `view_projection` without touching renderers.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    /// World-pixel position centered in the viewport (pan).
    pub center: [f32; 2],
    /// Zoom factor (1.0 = one world pixel per screen pixel).
    pub zoom: f32,
    /// Viewport size in pixels.
    pub viewport: [f32; 2],
}

impl Camera {
    pub fn new(viewport: [f32; 2]) -> Self {
        Self {
            center: [0.0, 0.0],
            zoom: 1.0,
            viewport,
        }
    }

    pub fn set_viewport(&mut self, viewport: [f32; 2]) {
        self.viewport = viewport;
    }

    pub fn set_center(&mut self, center: [f32; 2]) {
        self.center = center;
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.max(0.01);
    }

    /// Column-major 4x4 matrix mapping world-pixel coordinates to clip space.
    ///
    /// world -> (translate by -center) -> (scale by zoom) -> (ortho to clip).
    pub fn view_projection(&self) -> [[f32; 4]; 4] {
        let w = self.viewport[0].max(1.0);
        let h = self.viewport[1].max(1.0);

        // Combined scale: world pixels -> clip units. y is flipped because clip
        // space is y-up while world pixels are y-down.
        let sx = 2.0 * self.zoom / w;
        let sy = -2.0 * self.zoom / h;

        // Translation places `center` at clip origin.
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
