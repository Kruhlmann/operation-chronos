use core::time::Duration;

pub const CAMERA_MIN_ZOOM: f32 = 0.25;
pub const CAMERA_MAX_ZOOM: f32 = 4.0;
pub const CAMERA_ZOOM_STEP: f32 = 0.1;

pub const PAN_DRAG_TOLERANCE_PIXELS: f32 = 12.0;
pub const PAN_DRAG_CLICK_TIME: Duration = Duration::from_millis(200);
pub const PAN_DRAG_CLICK_TOLERANCE_PIXELS: f32 = 16.0;
