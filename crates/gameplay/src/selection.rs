pub type EntityId = u64;

#[derive(Clone, Copy, Debug)]
pub struct Marquee {
    pub origin: [f32; 2],
    pub current: [f32; 2],
}

impl Marquee {
    pub fn min(&self) -> [f32; 2] {
        [
            self.origin[0].min(self.current[0]),
            self.origin[1].min(self.current[1]),
        ]
    }

    pub fn max(&self) -> [f32; 2] {
        [
            self.origin[0].max(self.current[0]),
            self.origin[1].max(self.current[1]),
        ]
    }
}

#[derive(Default)]
pub struct Selection {
    pub marquee: Option<Marquee>,
}

#[derive(Clone, Copy, Debug)]
pub struct Selected;
