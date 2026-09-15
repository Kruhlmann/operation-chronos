use glam::Vec2;

use crate::constants::{
    GRASS_TOP_SPRITES, ISOMETRIC_TILE_HALF_HEIGHT, ISOMETRIC_TILE_HALF_WIDTH, ROCK_TOP_SPRITE,
    TILE_BASE_SPRITE,
};
use crate::geometry::{Disc, Position};

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
        matches!(self, Tile::Grass { .. })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WorldBounds {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl WorldBounds {
    pub fn center(&self) -> [f32; 2] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
        ]
    }
}

pub struct TilePlacement {
    pub world_pos: [f32; 2],
    pub sprite_index: u16,
}

pub struct Map {
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<Tile>,
}

impl Map {
    pub fn tile_to_world(x: u16, y: u16) -> [f32; 2] {
        let fx = x as f32;
        let fy = y as f32;
        [
            (fx - fy) * ISOMETRIC_TILE_HALF_WIDTH,
            (fx + fy) * ISOMETRIC_TILE_HALF_HEIGHT,
        ]
    }

    pub fn get_world_tile_at(p: Vec2) -> (i32, i32) {
        //   ( (tx-ty)*HALF_W, (tx+ty)*HALF_H + HALF_H )
        let a = p.x / ISOMETRIC_TILE_HALF_WIDTH; //  tx - ty
        let b = (p.y - ISOMETRIC_TILE_HALF_HEIGHT) / ISOMETRIC_TILE_HALF_HEIGHT; //  tx + ty
        let fx = (a + b) * 0.5;
        let fy = (b - a) * 0.5;
        (fx.round() as i32, fy.round() as i32)
    }

    pub fn tile_at(&self, tx: i32, ty: i32) -> Option<&Tile> {
        if tx < 0 || ty < 0 || tx >= self.width as i32 || ty >= self.height as i32 {
            return None;
        }
        let idx = ty as usize * self.width as usize + tx as usize;
        self.tiles.get(idx)
    }

    pub fn is_passable(&self, tx: i32, ty: i32) -> bool {
        self.tile_at(tx, ty).map(Tile::is_passable).unwrap_or(false)
    }

    pub fn is_passable_world(&self, p: Position) -> bool {
        let (tx, ty) = Self::get_world_tile_at(p.0);
        self.is_passable(tx, ty)
    }

    pub fn is_area_passable(&self, disc: Disc) -> bool {
        disc.axis_probes()
            .iter()
            .all(|c| self.is_passable_world(*c))
    }

    pub fn world_bounds(&self) -> WorldBounds {
        if self.width == 0 || self.height == 0 {
            return WorldBounds {
                min: [0.0, 0.0],
                max: [0.0, 0.0],
            };
        }
        let w = self.width - 1;
        let h = self.height - 1;
        let min_x = -(h as f32) * ISOMETRIC_TILE_HALF_WIDTH;
        let max_x = (w as f32) * ISOMETRIC_TILE_HALF_WIDTH;
        let min_y = 0.0;
        let max_y = (w as f32 + h as f32) * ISOMETRIC_TILE_HALF_HEIGHT;
        WorldBounds {
            min: [min_x, min_y],
            max: [max_x, max_y],
        }
    }

    pub fn placements(&self) -> Vec<TilePlacement> {
        let mut out: Vec<(u16, usize, TilePlacement)> = Vec::new();

        for (i, tile) in self.tiles.iter().enumerate() {
            let x = (i as u16) % self.width;
            let y = (i as u16) / self.width;
            let depth = x + y;
            let world_pos = Self::tile_to_world(x, y);

            for (layer, sprite_index) in tile.sprite_layers().into_iter().enumerate() {
                out.push((
                    depth,
                    layer,
                    TilePlacement {
                        world_pos,
                        sprite_index,
                    },
                ));
            }
        }

        out.sort_by_key(|(depth, layer, _)| (*depth, *layer));
        out.into_iter().map(|(_, _, p)| p).collect()
    }
}
