#[derive(Debug, Clone)]
pub enum Sprite {
    Animated {
        sprites: Vec<u16>,
        frame_duration: f32,
    },
    Static(u16),
}
