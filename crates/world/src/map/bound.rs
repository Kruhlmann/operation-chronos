#[derive(Debug, Clone, Copy)]
pub struct WorldBounds {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl WorldBounds {
    pub fn get_center(&self) -> [f32; 2] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
        ]
    }
}
