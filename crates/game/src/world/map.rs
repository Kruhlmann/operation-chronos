use crate::constants::{
    GRASS_TOP_SPRITES, ISO_TILE_HALF_HEIGHT, ISO_TILE_HALF_WIDTH, TILE_BASE_SPRITE,
};

pub enum Tile {
    Void,
    /// A grass tile. `variant` selects one of the grass top overlays.
    Grass {
        variant: u8,
    },
}

impl Tile {
    /// Sprite layers drawn for this tile, back-most first. Each tile draws a
    /// base block, then a decorative top on top of it. Returns an empty slice
    /// for tiles that are not drawn at all.
    pub fn sprite_layers(&self) -> Vec<u16> {
        match self {
            Tile::Void => Vec::new(),
            Tile::Grass { variant } => {
                let top = GRASS_TOP_SPRITES[(*variant as usize) % GRASS_TOP_SPRITES.len()];
                vec![TILE_BASE_SPRITE, top]
            }
        }
    }
}

/// A single drawable sprite layer placed in world-pixel space.
pub struct TilePlacement {
    /// World-pixel position of the layer's anchor.
    pub world_pos: [f32; 2],
    pub sprite_index: u16,
}

pub struct Map {
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<Tile>,
}

impl Map {
    /// Project tile grid coordinates to world-pixel space using an isometric
    /// (diamond) layout.
    pub fn tile_to_world(x: u16, y: u16) -> [f32; 2] {
        let fx = x as f32;
        let fy = y as f32;
        [
            (fx - fy) * ISO_TILE_HALF_WIDTH,
            (fx + fy) * ISO_TILE_HALF_HEIGHT,
        ]
    }

    /// Drawable sprite layers in world-pixel space, sorted back-to-front so
    /// nearer tiles overdraw farther ones (painter's algorithm). Within a
    /// single tile, layers keep their declared order (base before top).
    pub fn placements(&self) -> Vec<TilePlacement> {
        // (tile_depth, layer_within_tile, placement) — a stable sort by the
        // first two keys keeps base-before-top ordering per tile.
        let mut out: Vec<(u16, usize, TilePlacement)> = Vec::new();

        for (i, tile) in self.tiles.iter().enumerate() {
            let x = (i as u16) % self.width;
            let y = (i as u16) / self.width;
            let world_pos = Self::tile_to_world(x, y);
            // Depth key: larger x+y is drawn later (in front).
            let depth = x + y;

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
