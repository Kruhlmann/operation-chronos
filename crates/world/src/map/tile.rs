use data::constants::{GRASS_TOP_SPRITES, ROCK_TOP_SPRITE, TILE_BASE_SPRITE};

pub struct TilePlacement {
    pub world_position: [f32; 2],
    pub sprite_index: u16,
}

pub type TilePosition = (i32, i32);

#[derive(serde::Deserialize, serde::Serialize)]
pub enum Tile {
    Void,
    Grass { variant: u8 },
    Rock,
}

impl Tile {
    pub fn sprite_layers(&self) -> Vec<u16> {
        match self {
            Tile::Void => Vec::new(),
            Tile::Grass { variant } => {
                let top = GRASS_TOP_SPRITES[(*variant as usize) % GRASS_TOP_SPRITES.len()];
                vec![TILE_BASE_SPRITE, top]
            }
            Tile::Rock => vec![TILE_BASE_SPRITE, ROCK_TOP_SPRITE],
        }
    }

    pub fn is_passable(&self) -> bool {
        match self {
            Tile::Void => false,
            Tile::Grass { .. } => true,
            Tile::Rock => false,
        }
    }
}
