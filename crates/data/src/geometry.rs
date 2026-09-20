use core::ops::{Add, AddAssign, Sub, SubAssign};

use crate::constants::ISOMETRIC_GROUND_SQUASH;
pub use crate::constants::ISOMETRIC_GROUND_SQUASH as RENDER_GROUND_SQUASH;
use crate::math::{FacingVec2, FixedVec2, Scalar, scalar_from_render, scalar_sqrt};

pub const GROUND_SQUASH: Scalar = Scalar::const_from_int(1).strict_div_int(2);

const _: () = {
    assert!(ISOMETRIC_GROUND_SQUASH == 0.5);
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Position(pub FixedVec2);

impl Position {
    #[inline]
    pub fn new(x: Scalar, y: Scalar) -> Self {
        Self(FixedVec2::new(x, y))
    }

    #[inline]
    pub fn from_render(v: glam::Vec2) -> Self {
        Self(FixedVec2::from_render(v))
    }

    #[inline]
    pub fn to_render(self) -> glam::Vec2 {
        self.0.to_render()
    }

    #[inline]
    pub fn is_in_bounds(&self, min: FixedVec2, max: FixedVec2) -> bool {
        self.0.x >= min.x && self.0.x <= max.x && self.0.y >= min.y && self.0.y <= max.y
    }

    #[inline]
    pub fn hits(&self, world_point: Position, radius: Scalar) -> bool {
        Disc::new(*self, radius).contains(world_point)
    }
}

impl Sub for Position {
    type Output = FixedVec2;
    #[inline]
    fn sub(self, rhs: Position) -> FixedVec2 {
        self.0 - rhs.0
    }
}

impl Add<FixedVec2> for Position {
    type Output = Position;
    #[inline]
    fn add(self, rhs: FixedVec2) -> Position {
        Position(self.0 + rhs)
    }
}

impl Sub<FixedVec2> for Position {
    type Output = Position;
    #[inline]
    fn sub(self, rhs: FixedVec2) -> Position {
        Position(self.0 - rhs)
    }
}

impl AddAssign<FixedVec2> for Position {
    #[inline]
    fn add_assign(&mut self, rhs: FixedVec2) {
        self.0 += rhs;
    }
}

impl SubAssign<FixedVec2> for Position {
    #[inline]
    fn sub_assign(&mut self, rhs: FixedVec2) {
        self.0 -= rhs;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Facing(pub FacingVec2);

#[derive(Clone, Copy, Debug)]
pub struct Footprint {
    pub radius: Scalar,
    pub offset: FixedVec2,
}

impl Footprint {
    #[inline]
    pub fn new(radius: Scalar) -> Self {
        Self {
            radius,
            offset: FixedVec2::ZERO,
        }
    }

    #[inline]
    pub fn with_offset(radius: Scalar, offset: FixedVec2) -> Self {
        Self { radius, offset }
    }

    #[inline]
    pub fn disc_at(&self, position: Position) -> Disc {
        Disc::new(position + self.offset, self.radius)
    }
}

#[inline]
fn world_to_ground(offset: FixedVec2) -> FixedVec2 {
    FixedVec2::new(offset.x, offset.y / (Scalar::ONE / GROUND_SQUASH))
}

#[inline]
fn ground_to_world(offset: FixedVec2) -> FixedVec2 {
    FixedVec2::new(offset.x, offset.y * (Scalar::ONE / GROUND_SQUASH))
}

#[derive(Clone, Copy, Debug)]
pub struct Disc {
    pub center: Position,
    pub radius: Scalar,
}

impl Disc {
    #[inline]
    pub fn new(center: Position, radius: Scalar) -> Self {
        Self { center, radius }
    }

    #[inline]
    pub fn distance_to(&self, world_point: Position) -> Scalar {
        world_to_ground(world_point - self.center).length()
    }

    #[inline]
    pub fn contains(&self, world_point: Position) -> bool {
        world_to_ground(world_point - self.center).length_squared() <= self.radius * self.radius
    }

    pub fn separation(&self, other: &Disc) -> Option<FixedVec2> {
        let delta = world_to_ground(other.center - self.center);
        let min_dist = self.radius + other.radius;
        if delta.x.abs() >= min_dist || delta.y.abs() >= min_dist {
            return None;
        }
        let dist_sq = delta.length_squared();
        if dist_sq >= min_dist * min_dist {
            return None;
        }
        let dist = scalar_sqrt(dist_sq);
        let (normal, overlap) = if dist > Scalar::from_bits(64) {
            (delta / dist, min_dist - dist)
        } else {
            (FixedVec2::X, min_dist)
        };
        Some(ground_to_world(normal * overlap))
    }

    #[inline]
    pub fn screen_radii_render(&self) -> glam::Vec2 {
        let r = self.radius.to_num::<f32>();
        glam::Vec2::new(r, r * ISOMETRIC_GROUND_SQUASH)
    }

    #[inline]
    fn screen_radii(&self) -> FixedVec2 {
        FixedVec2::new(self.radius, self.radius * GROUND_SQUASH)
    }

    pub fn axis_probes(&self) -> [Position; 5] {
        let r = self.screen_radii();
        [
            self.center,
            self.center + FixedVec2::new(-r.x, Scalar::ZERO),
            self.center + FixedVec2::new(r.x, Scalar::ZERO),
            self.center + FixedVec2::new(Scalar::ZERO, -r.y),
            self.center + FixedVec2::new(Scalar::ZERO, r.y),
        ]
    }

    pub fn outline_render(&self, segments: usize) -> impl Iterator<Item = glam::Vec2> + '_ {
        let r = self.screen_radii_render();
        let center = self.center.to_render();
        (0..segments).map(move |i| {
            let t = (i as f32) / (segments as f32) * std::f32::consts::TAU;
            center + glam::Vec2::new(r.x * t.cos(), r.y * t.sin())
        })
    }
}

#[inline]
pub fn scalar_from_world_f32(value: f32) -> Scalar {
    scalar_from_render(value)
}
