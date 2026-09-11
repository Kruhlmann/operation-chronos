pub enum Tile {
    Void,
    Grass,
}

impl Tile {
    pub fn sprite_index(&self) -> Option<u16> {
        match self {
            Tile::Void => None,
            Tile::Grass => Some(0),
        }
    }
}

pub struct Map {
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<Tile>,
}

impl Map {
    pub fn drawable_tiles(&self) -> impl Iterator<Item = (u16, u16, u16)> + '_ {
        self.tiles.iter().enumerate().filter_map(move |(i, tile)| {
            let sprite = tile.sprite_index()?;
            let x = (i as u16) % self.width;
            let y = (i as u16) / self.width;
            Some((x, y, sprite))
        })
    }
}
