use core::time::Duration;

#[macro_export]
macro_rules! sprite_index {
    ($row:expr, $col:expr) => {
        $row * $crate::constants::SPRITE_SHEET_COLUMNS + $col
    };
}

pub const ASSET_DIRECTORY: &str = "res";
pub const SPRITE_SHEET_PATH: &str = "world.png";
pub const SPRITE_SHEET_COLUMNS: u16 = 16;
pub const SPRITE_SHEET_ROWS: u16 = 16;
pub const SPRITE_SIZE_PIXELS: u16 = 64;
pub const GRASS_TOP_SPRITES: [u16; 3] = [
    sprite_index!(1, 0),
    sprite_index!(1, 1),
    sprite_index!(1, 2),
];
pub const TILE_BASE_SPRITE: u16 = sprite_index!(0, 0);
pub const ROCK_TOP_SPRITE: u16 = sprite_index!(3, 0);

pub const DEFAULT_MAP_WIDTH: u16 = 64;
pub const DEFAULT_MAP_HEIGHT: u16 = 64;
pub const MAX_WORLD_TILES: u64 = 65_536;
pub const ISO_TILE_HALF_WIDTH: f32 = 32.0;
pub const ISO_TILE_HALF_HEIGHT: f32 = 16.0;

pub const CAMERA_MIN_ZOOM: f32 = 0.25;
pub const CAMERA_MAX_ZOOM: f32 = 4.0;
pub const CAMERA_ZOOM_STEP: f32 = 0.1;

// Pixel tolerance for when to transition from considering mouse down a "Click" versus a "Drag"
pub const PAN_DRAG_TOLERANCE: f32 = 12.0;
pub const PAN_DRAG_CLICK_TIME: Duration = Duration::from_millis(200);
pub const PAN_DRAG_CLICK_TOLERANCE_PIXELS: f32 = 16.0;
pub const ORDER_MARKER_RENDER_DURATION: Duration = Duration::from_millis(600);

pub const FONT_GLYPH_SIZE: u32 = 8;
pub const FONT_MAX_CHARACTERS: usize = 256;
pub const FONT_CHARSET: &[u8] = b" .:0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub const DEBUG_PRAGMA: &str = "\
    wgpu=info,\
    naga=info,\
    winit=info,\
    winit::platform_impl::linux::x11::xdisplay=error,\
    wgpu_hal::vulkan::instance=error,\
    wgpu_hal::vulkan::adapter=warn,\
    winit::platform_impl::linux::x11::window=warn,\
    debug";
pub const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / 60);
pub const TICK_TIME: Duration = Duration::from_nanos(1_000_000_000 / 60);
