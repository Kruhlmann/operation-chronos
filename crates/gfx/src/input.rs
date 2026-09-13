use std::time::Instant;

use world::World;
use world::constants::{
    CAMERA_ZOOM_STEP, PAN_DRAG_CLICK_TIME, PAN_DRAG_CLICK_TOLERANCE_PIXELS, PAN_DRAG_TOLERANCE,
};

use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};

#[derive(Default)]
pub struct Input {
    cursor: [f32; 2],
    active: Option<ActiveDrag>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Button {
    Left,
    Right,
}

struct ActiveDrag {
    button: Button,
    drag: Drag,
}

struct Drag {
    origin: [f32; 2],
    last_pos: [f32; 2],
    active: bool,
    pressed_at: Instant,
}

impl Drag {
    fn new(pos: [f32; 2]) -> Self {
        Self {
            origin: pos,
            last_pos: pos,
            active: false,
            pressed_at: Instant::now(),
        }
    }

    fn manhatten_travel_distance(&self, pos: [f32; 2]) -> f32 {
        (pos[0] - self.origin[0]).abs() + (pos[1] - self.origin[1]).abs()
    }

    fn in_click_grace(&self, pos: [f32; 2]) -> bool {
        self.pressed_at.elapsed() < PAN_DRAG_CLICK_TIME
            && self.manhatten_travel_distance(pos) < PAN_DRAG_CLICK_TOLERANCE_PIXELS
    }

    fn update(&mut self, pos: [f32; 2]) -> Option<[f32; 2]> {
        let delta = [pos[0] - self.last_pos[0], pos[1] - self.last_pos[1]];
        self.last_pos = pos;
        if !self.active {
            // Never activate drag inside the grace window; a quick release
            // there always wins as a click.
            if self.in_click_grace(pos) {
                return None;
            }
            if self.manhatten_travel_distance(pos) < PAN_DRAG_TOLERANCE {
                return None;
            }
            self.active = true;
        }
        Some(delta)
    }
}

pub enum InputEventSideEffect {
    UpdateCamera,
    ClickLeft([f32; 2]),
    ClickRight([f32; 2]),
}

impl Input {
    pub fn handle(
        &mut self,
        event: &WindowEvent,
        world: &mut World,
    ) -> Option<InputEventSideEffect> {
        match event {
            WindowEvent::MouseInput {
                button,
                state: ElementState::Pressed,
                ..
            } => {
                if self.active.is_none() {
                    let button = match button {
                        MouseButton::Left => Some(Button::Left),
                        MouseButton::Right => Some(Button::Right),
                        _ => None,
                    };
                    if let Some(button) = button {
                        self.active = Some(ActiveDrag {
                            button,
                            drag: Drag::new(self.cursor),
                        });
                    }
                }
                None
            }
            WindowEvent::MouseInput {
                button,
                state: ElementState::Released,
                ..
            } => {
                let released = match button {
                    MouseButton::Left => Button::Left,
                    MouseButton::Right => Button::Right,
                    _ => return None,
                };
                match &self.active {
                    Some(a) if a.button == released => {
                        let a = self.active.take().unwrap();
                        // Grace window forces a click regardless of drag state.
                        let click_wins = a.drag.in_click_grace(self.cursor);
                        match a.button {
                            Button::Left if click_wins => {
                                world.clear_marquee();
                                Some(InputEventSideEffect::ClickLeft(self.cursor))
                            }
                            Button::Left if a.drag.active => {
                                world.commit_marquee();
                                None
                            }
                            Button::Left => Some(InputEventSideEffect::ClickLeft(self.cursor)),
                            Button::Right if click_wins || !a.drag.active => {
                                Some(InputEventSideEffect::ClickRight(self.cursor))
                            }
                            Button::Right => None,
                        }
                    }
                    _ => None,
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = [position.x as f32, position.y as f32];
                let a = self.active.as_mut()?;
                let delta = a.drag.update(self.cursor)?;
                match a.button {
                    Button::Right => {
                        world.pan(-delta[0], -delta[1]);
                        Some(InputEventSideEffect::UpdateCamera)
                    }
                    Button::Left => {
                        world.set_marquee(a.drag.origin, self.cursor);
                        None
                    }
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
                    Some(InputEventSideEffect::UpdateCamera)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
