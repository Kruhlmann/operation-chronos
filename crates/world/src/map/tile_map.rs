use glam::Vec2;

use data::constants::{ISOMETRIC_TILE_HALF_HEIGHT, ISOMETRIC_TILE_HALF_WIDTH};
use data::geometry::{Disc, Position};

use crate::{Tile, TilePlacement, TilePosition, WorldBounds};

pub struct Map {
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<Tile>,
}

#[derive()]
pub enum MapInitError {
    MapSizeIncorrect(u16, u16, usize),
}

impl Map {
    pub fn new(width: u16, height: u16, tiles: Vec<Tile>) -> Result<Self, MapInitError> {
        if tiles.len() != (width as usize) * (height as usize) {
            return Err(MapInitError::MapSizeIncorrect(width, height, tiles.len()));
        }
        Ok(Map {
            width,
            height,
            tiles,
        })
    }

    pub fn tile_to_world(x: u16, y: u16) -> [f32; 2] {
        let fx = x as f32;
        let fy = y as f32;
        [
            (fx - fy) * ISOMETRIC_TILE_HALF_WIDTH,
            (fx + fy) * ISOMETRIC_TILE_HALF_HEIGHT,
        ]
    }

    pub fn get_tile_neighbors_unchecked(&self, tile: TilePosition) -> [TilePosition; 4] {
        [
            (tile.0 + 1, tile.1),
            (tile.0 - 1, tile.1),
            (tile.0, tile.1 + 1),
            (tile.0, tile.1 - 1),
        ]
    }

    pub fn find_nearest_passable_tile_in_radius(
        &self,
        goal: TilePosition,
        radius: i32,
    ) -> Option<TilePosition> {
        for r in 1..=radius {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    let n = (goal.0 + dx, goal.1 + dy);
                    if self.is_passable(n.0, n.1) {
                        return Some(n);
                    }
                }
            }
        }
        None
    }

    pub fn get_world_tile_at(p: Vec2) -> TilePosition {
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
                        world_position: world_pos,
                        sprite_index,
                    },
                ));
            }
        }

        out.sort_by_key(|(depth, layer, _)| (*depth, *layer));
        out.into_iter().map(|(_, _, p)| p).collect()
    }
}
