pub const SPRITE_SHEET_COLUMNS: u16 = 16;
pub const SPRITE_SHEET_ROWS: u16 = 16;
#[macro_export]
macro_rules! sprite_index {
    ($row:expr, $col:expr) => {
        $row * $crate::constants::SPRITE_SHEET_COLUMNS + $col
    };
}

pub const ASSET_DIRECTORY: &str = "res";
pub const SPRITE_SHEET_PATH: &str = "world.png";
pub const SPRITE_SIZE_PIXELS: u16 = 64;
pub const GRASS_TOP_SPRITES: [u16; 3] = [
    sprite_index!(1, 0),
    sprite_index!(1, 1),
    sprite_index!(1, 2),
];
pub const TILE_BASE_SPRITE: u16 = sprite_index!(0, 0);
pub const ROCK_TOP_SPRITE: u16 = sprite_index!(3, 0);
pub const TILE_SPRITE_ANCHOR: [f32; 2] = [31.0, 23.0];

pub const FONT_GLYPH_SIZE: u32 = 8;
pub const FONT_MAX_CHARACTERS: usize = 256;
pub const FONT_CHARSET: &[u8] = b" .:0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
