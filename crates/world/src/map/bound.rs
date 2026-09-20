use data::math::{FixedVec2, Scalar, scalar_from_render};

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

    #[inline]
    pub fn min_sim(&self) -> FixedVec2 {
        FixedVec2::new(
            scalar_from_render(self.min[0]),
            scalar_from_render(self.min[1]),
        )
    }

    #[inline]
    pub fn max_sim(&self) -> FixedVec2 {
        FixedVec2::new(
            scalar_from_render(self.max[0]),
            scalar_from_render(self.max[1]),
        )
    }

    #[inline]
    pub fn center_sim(&self) -> FixedVec2 {
        let half = Scalar::const_from_int(1).strict_div_int(2);
        FixedVec2::new(
            (scalar_from_render(self.min[0]) + scalar_from_render(self.max[0])) * half,
            (scalar_from_render(self.min[1]) + scalar_from_render(self.max[1])) * half,
        )
    }
}
