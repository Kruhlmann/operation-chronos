use crate::math::scalar::SCALAR_EPSILON;
use crate::math::vec2::FixedVec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FacingVec2(FixedVec2);

impl FacingVec2 {
    pub const EAST: Self = Self(FixedVec2::X);

    #[inline]
    pub fn from_direction(delta: FixedVec2) -> Option<Self> {
        let len = delta.length();
        if len <= SCALAR_EPSILON {
            return None;
        }
        Some(Self(delta / len))
    }

    #[inline]
    pub fn as_vec(self) -> FixedVec2 {
        self.0
    }

    #[inline]
    pub fn to_angle_render(self) -> f32 {
        let v = self.0.to_render();
        v.y.atan2(v.x)
    }
}

impl Default for FacingVec2 {
    #[inline]
    fn default() -> Self {
        Self::EAST
    }
}
