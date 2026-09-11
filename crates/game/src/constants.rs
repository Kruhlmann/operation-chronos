use core::time::Duration;

pub const ASSET_DIRECTORY: &str = "res";
pub const SPRITE_SHEET_PATH: &str = "sprites.png";
pub const SPRITE_SHEET_COLUMNS: u16 = 16;
pub const SPRITE_SHEET_ROWS: u16 = 16;
pub const SPRITE_SIZE_PIXELS: u16 = 64;
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
