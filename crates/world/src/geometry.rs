use core::ops::{Add, AddAssign, Sub, SubAssign};

use glam::Vec2;

use crate::constants::ISOMETRIC_GROUND_SQUASH;

/// An affine point in world-pixel space. Subtracting two `Position`s yields a
/// `Vec2` displacement; adding a `Vec2` displacement to a `Position` yields a
/// `Position`. This keeps points and displacements from being conflated.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position(pub Vec2);

impl Position {
    pub fn is_in_bounds(&self, min: Vec2, max: Vec2) -> bool {
        self.0.x >= min.x && self.0.x <= max.x && self.0.y >= min.y && self.0.y <= max.y
    }

    pub fn hits(&self, world_point: Position, radius: f32) -> bool {
        Disc::new(*self, radius).contains(world_point)
    }
}

impl Sub for Position {
    type Output = Vec2;
    #[inline]
    fn sub(self, rhs: Position) -> Vec2 {
        self.0 - rhs.0
    }
}

impl Add<Vec2> for Position {
    type Output = Position;
    #[inline]
    fn add(self, rhs: Vec2) -> Position {
        Position(self.0 + rhs)
    }
}

impl Sub<Vec2> for Position {
    type Output = Position;
    #[inline]
    fn sub(self, rhs: Vec2) -> Position {
        Position(self.0 - rhs)
    }
}

impl AddAssign<Vec2> for Position {
    #[inline]
    fn add_assign(&mut self, rhs: Vec2) {
        self.0 += rhs;
    }
}

impl SubAssign<Vec2> for Position {
    #[inline]
    fn sub_assign(&mut self, rhs: Vec2) {
        self.0 -= rhs;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Facing(pub f32);

#[derive(Clone, Copy, Debug)]
pub struct Footprint {
    pub radius: f32,
    pub offset: Vec2,
}

impl Footprint {
    #[inline]
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            offset: Vec2::ZERO,
        }
    }

    #[inline]
    pub fn with_offset(radius: f32, offset: Vec2) -> Self {
        Self { radius, offset }
    }

    #[inline]
    pub fn disc_at(&self, position: Position) -> Disc {
        Disc::new(position + self.offset, self.radius)
    }
}

#[inline]
fn world_to_ground(offset: Vec2) -> Vec2 {
    Vec2::new(offset.x, offset.y / ISOMETRIC_GROUND_SQUASH)
}

#[inline]
fn ground_to_world(offset: Vec2) -> Vec2 {
    Vec2::new(offset.x, offset.y * ISOMETRIC_GROUND_SQUASH)
}

#[derive(Clone, Copy, Debug)]
pub struct Disc {
    pub center: Position,
    pub radius: f32,
}

impl Disc {
    #[inline]
    pub fn new(center: Position, radius: f32) -> Self {
        Self { center, radius }
    }

    #[inline]
    pub fn distance_to(&self, world_point: Position) -> f32 {
        world_to_ground(world_point - self.center).length()
    }

    #[inline]
    pub fn contains(&self, world_point: Position) -> bool {
        world_to_ground(world_point - self.center).length_squared() <= self.radius * self.radius
    }

    pub fn separation(&self, other: &Disc) -> Option<Vec2> {
        let delta = world_to_ground(other.center - self.center);
        let min_dist = self.radius + other.radius;
        let dist_sq = delta.length_squared();
        if dist_sq >= min_dist * min_dist {
            return None;
        }
        let dist = dist_sq.sqrt();
        let (normal, overlap) = if dist > 1e-4 {
            (delta / dist, min_dist - dist)
        } else {
            (Vec2::X, min_dist)
        };
        Some(ground_to_world(normal * overlap))
    }

    #[inline]
    pub fn screen_radii(&self) -> Vec2 {
        Vec2::new(self.radius, self.radius * ISOMETRIC_GROUND_SQUASH)
    }

    pub fn axis_probes(&self) -> [Position; 5] {
        let r = self.screen_radii();
        [
            self.center,
            self.center + Vec2::new(-r.x, 0.0),
            self.center + Vec2::new(r.x, 0.0),
            self.center + Vec2::new(0.0, -r.y),
            self.center + Vec2::new(0.0, r.y),
        ]
    }

    pub fn outline(&self, segments: usize) -> impl Iterator<Item = Position> + '_ {
        let r = self.screen_radii();
        (0..segments).map(move |i| {
            let t = (i as f32) / (segments as f32) * std::f32::consts::TAU;
            self.center + Vec2::new(r.x * t.cos(), r.y * t.sin())
        })
    }
}
