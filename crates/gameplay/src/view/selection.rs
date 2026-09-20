#[derive(Clone, Copy, Debug)]
pub struct SelectionMarquee {
    pub origin: [f32; 2],
    pub current: [f32; 2],
}

impl SelectionMarquee {
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
    pub marquee: Option<SelectionMarquee>,
    pub selected: Vec<hecs::Entity>,
}

impl Selection {
    pub fn clear(&mut self) {
        self.selected.clear();
    }

    pub fn replace(&mut self, entities: Vec<hecs::Entity>) {
        self.selected = entities;
    }

    pub fn contains(&self, entity: hecs::Entity) -> bool {
        self.selected.contains(&entity)
    }
}
