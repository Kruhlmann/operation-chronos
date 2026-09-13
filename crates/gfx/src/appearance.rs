use glam::Vec2;

use world::Facing;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SheetId {
    TankBody,
    TankTurret,
}

impl SheetId {
    pub const ALL: &'static [SheetId] = &[SheetId::TankBody, SheetId::TankTurret];

    pub fn asset_path(&self) -> &'static str {
        match self {
            SheetId::TankBody => "res/unit/double-barrel-tank/body.png",
            SheetId::TankTurret => "res/unit/double-barrel-tank/turret.png",
        }
    }

    pub fn frame_size(&self) -> (f32, f32) {
        match self {
            SheetId::TankBody => (70.0, 48.0),
            SheetId::TankTurret => (62.0, 42.0),
        }
    }

    pub fn facings(&self) -> u32 {
        match self {
            SheetId::TankBody => 32,
            SheetId::TankTurret => 32,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpriteRef {
    pub sheet: SheetId,
    pub frame_base: u32,
    pub frame_count: u32,
}

impl SpriteRef {
    pub fn frame_for(&self, facing: Facing) -> u32 {
        let total = self.sheet.facings().max(1);
        let count = self.frame_count.max(1);
        let stride = (total / count).max(1);

        let two_pi = std::f32::consts::TAU;
        let mut a = facing.0 % two_pi;
        if a < 0.0 {
            a += two_pi;
        }
        let dir = ((a / two_pi) * count as f32).round() as u32 % count;
        (self.frame_base + dir * stride) % total
    }
}

#[derive(Clone, Debug)]
pub struct SpriteParts(pub Vec<SpritePart>);

#[derive(Clone, Copy, Debug)]
pub struct SpritePart {
    pub sprite: SpriteRef,
    pub offset: Vec2,
    pub facing_override: Option<f32>,
}
