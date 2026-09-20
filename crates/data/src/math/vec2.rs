use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

use crate::math::scalar::{Scalar, scalar_from_render, scalar_sqrt, scalar_to_render};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FixedVec2 {
    pub x: Scalar,
    pub y: Scalar,
}

impl FixedVec2 {
    pub const ZERO: Self = Self {
        x: Scalar::ZERO,
        y: Scalar::ZERO,
    };

    pub const X: Self = Self {
        x: Scalar::ONE,
        y: Scalar::ZERO,
    };

    #[inline]
    pub const fn new(x: Scalar, y: Scalar) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn from_render(v: glam::Vec2) -> Self {
        Self::new(scalar_from_render(v.x), scalar_from_render(v.y))
    }

    #[inline]
    pub fn to_render(self) -> glam::Vec2 {
        glam::Vec2::new(scalar_to_render(self.x), scalar_to_render(self.y))
    }

    #[inline]
    pub fn dot(self, other: Self) -> Scalar {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn length_squared(self) -> Scalar {
        self.dot(self)
    }

    #[inline]
    pub fn length(self) -> Scalar {
        scalar_sqrt(self.length_squared())
    }

    #[inline]
    pub fn normalize_or_zero(self) -> Self {
        let len = self.length();
        if len <= Scalar::ZERO {
            Self::ZERO
        } else {
            self / len
        }
    }
}

impl Add for FixedVec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for FixedVec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Neg for FixedVec2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl AddAssign for FixedVec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign for FixedVec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<Scalar> for FixedVec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Scalar) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<Scalar> for FixedVec2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Scalar) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}
