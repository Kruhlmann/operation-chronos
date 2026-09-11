lazy_static::lazy_static! {
    pub static ref HUD: &'static str = include_str!("hud.wgsl");
    pub static ref HUD_RECT: &'static str = include_str!("hud_rect.wgsl");
    pub static ref WORLD: &'static str = include_str!("world.wgsl");
    pub static ref UNIT: &'static str = include_str!("unit.wgsl");
}
