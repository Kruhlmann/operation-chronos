use core::time::Duration;

#[macro_export]
macro_rules! sprite_index {
    ($row:expr, $col:expr) => {
        $row * $crate::constants::SPRITE_SHEET_COLUMNS + $col
    };
}

pub const ASSET_DIRECTORY: &str = "res";
pub const SPRITE_SHEET_PATH: &str = "sprites.png";
pub const SPRITE_SHEET_COLUMNS: u16 = 16;
pub const SPRITE_SHEET_ROWS: u16 = 16;
pub const SPRITE_SIZE_PIXELS: u16 = 64;
pub const ISO_TILE_HALF_WIDTH: f32 = 32.0;
pub const ISO_TILE_HALF_HEIGHT: f32 = 16.0;
pub const MAX_WORLD_TILES: u64 = 65_536;
pub const TILE_BASE_SPRITE: u16 = sprite_index!(0, 0);
pub const GRASS_TOP_SPRITES: [u16; 3] = [
    sprite_index!(1, 0),
    sprite_index!(1, 1),
    sprite_index!(1, 2),
];
pub const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / 60);
pub const FONT_GLYPH_SIZE: u32 = 8;
pub const FONT_MAX_CHARACTERS: usize = 256;
pub const FONT_CHARSET: &[u8] = b" .:0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const DEBUG_PRAGMA: &str = "\
    wgpu=info,\
    naga=info,\
    winit=info,\
    winit::platform_impl::linux::x11::xdisplay=error,\
    wgpu_hal::vulkan::instance=error,\
    debug";
