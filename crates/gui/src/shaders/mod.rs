lazy_static::lazy_static! {
    pub static ref HUD: &'static str = include_str!("hud.wgsl");
    pub static ref WORLD: &'static str = include_str!("world.wgsl");
}
