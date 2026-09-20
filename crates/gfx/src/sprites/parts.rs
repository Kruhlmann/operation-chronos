use data::math::FacingVec2;
use glam::Vec2;

use crate::SpriteRef;

#[derive(Clone, Debug)]
pub struct SpriteParts(pub Vec<SpritePart>);

#[derive(Clone, Copy, Debug)]
pub struct SpritePart {
    pub sprite: SpriteRef,
    pub offset: Vec2,
    pub facing_override: Option<FacingVec2>,
}
