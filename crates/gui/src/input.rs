use game::World;
use game::constants::CAMERA_ZOOM_STEP;

use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};

#[derive(Default)]
pub struct Input {
    cursor: [f32; 2],
    panning: bool,
    last_cursor: [f32; 2],
}

impl Input {
    pub fn handle(&mut self, event: &WindowEvent, world: &mut World) -> bool {
        match event {
            WindowEvent::MouseInput { button, state, .. } => {
                if *button == MouseButton::Right {
                    self.panning = *state == ElementState::Pressed;
                    self.last_cursor = self.cursor;
                }
                false
            }

            WindowEvent::CursorMoved { position, .. } => {
                let pos = [position.x as f32, position.y as f32];
                self.cursor = pos;
                if self.panning {
                    let dx = pos[0] - self.last_cursor[0];
                    let dy = pos[1] - self.last_cursor[1];
                    self.last_cursor = pos;
                    world.pan(dx, dy);
                    true
                } else {
                    false
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(p) => (p.y as f32) / 120.0,
                };
                if scroll != 0.0 {
                    let factor = 1.0 + scroll * CAMERA_ZOOM_STEP;
                    world.zoom_at(factor, self.cursor);
                    true
                } else {
                    false
                }
            }

            _ => false,
        }
    }
}
