use data::geometry::Facing;

use crate::SheetId;

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
        let mut a = facing.0.to_angle_render() % std::f32::consts::TAU;
        if a < 0.0 {
            a += std::f32::consts::TAU;
        }
        let dir = ((a / std::f32::consts::TAU) * count as f32).round() as u32 % count;
        (self.frame_base + dir * stride) % total
    }
}
