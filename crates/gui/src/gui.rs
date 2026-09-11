use std::sync::Arc;
use std::time::Instant;

use crate::{Gpu, HudRenderer, WorldRenderer};
use game::constants::{
    ASSET_DIRECTORY, FRAME_TIME, SPRITE_SHEET_COLUMNS, SPRITE_SHEET_PATH, SPRITE_SHEET_ROWS,
    SPRITE_SIZE_PIXELS,
};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

pub struct Running {
    pub window: Arc<Window>,
    pub gpu: Gpu,
    pub assets: game::GpuAssets,
    pub world: WorldRenderer,
    pub camera: game::Camera,
    pub hud: HudRenderer,
}

pub enum State {
    Uninitialized,
    Running(Running),
}

pub struct Gui {
    pub state: State,
    pub library: game::AssetLibrary,
    pub next_frame_scheduled: Instant,
    pub initial_logical_size: [u32; 2],
    fps_last_frame: Instant,
    avg_frame_time: f32,
    hud_last_update: Instant,
}

impl Gui {
    pub fn new(library: game::AssetLibrary) -> Self {
        Self {
            state: State::Uninitialized,
            library,
            next_frame_scheduled: Instant::now(),
            initial_logical_size: [800, 600],
            fps_last_frame: Instant::now(),
            avg_frame_time: 0.0,
            hud_last_update: Instant::now(),
        }
    }
}

impl ApplicationHandler for Gui {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if matches!(self.state, State::Running(_)) {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("wgpu example")
                        .with_inner_size(winit::dpi::LogicalSize::new(
                            self.initial_logical_size[0],
                            self.initial_logical_size[1],
                        )),
                )
                .expect("failed to create window"),
        );

        let gpu = pollster::block_on(Gpu::new(window.clone()));

        let assets = game::GpuAssets::load(&gpu.device, &gpu.queue, &self.library)
            .expect("failed to load GPU assets");

        let sprite_sheet_path = format!("{ASSET_DIRECTORY}/{SPRITE_SHEET_PATH}");
        let sprite_sheet = assets
            .get(&sprite_sheet_path)
            .unwrap_or_else(|| panic!("sprite sheet not loaded: {sprite_sheet_path}"));

        let surface_size = [gpu.config.width as f32, gpu.config.height as f32];
        let mut hud = HudRenderer::new(
            &gpu.device,
            &gpu.queue,
            gpu.config.format,
            surface_size,
            2.0,
        );
        hud.set_text(&gpu.queue, "FPS: 0", [8.0, 8.0]);

        // Build the world renderer and a placeholder map until the real world
        // exists.
        let map = placeholder_map();

        // Center the camera on the middle of the map (in world-pixel space).
        let mut camera = game::Camera::new(surface_size);
        let mid = game::Map::tile_to_world(map.width / 2, map.height / 2);
        camera.set_center(mid);

        let mut world = WorldRenderer::new(
            &gpu.device,
            &gpu.queue,
            gpu.config.format,
            sprite_sheet,
            SPRITE_SHEET_COLUMNS,
            SPRITE_SHEET_ROWS,
            SPRITE_SIZE_PIXELS as f32,
            &camera,
        );
        world.set_map(&gpu.queue, &map);

        self.state = State::Running(Running {
            window,
            gpu,
            assets,
            world,
            camera,
            hud,
        });
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        if now >= self.next_frame_scheduled {
            if let State::Running(running) = &self.state {
                running.window.request_redraw();
            }

            self.next_frame_scheduled = now + FRAME_TIME;
        }

        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            self.next_frame_scheduled,
        ));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let State::Running(running) = &self.state {
            if running.window.id() != window_id {
                return;
            }
        } else {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                let State::Running(running) = &mut self.state else {
                    return;
                };
                running.gpu.resize(size.width, size.height);
                if size.height > 0 {
                    running.hud.set_surface_size(
                        &running.gpu.queue,
                        [size.width as f32, size.height as f32],
                    );
                    running
                        .camera
                        .set_viewport([size.width as f32, size.height as f32]);
                    running
                        .world
                        .set_camera(&running.gpu.queue, &running.camera);
                }
            }

            WindowEvent::RedrawRequested => {
                // Measure this frame's delta and fold it into an exponential
                // moving average for smooth readings.
                let now = Instant::now();
                let dt = now.duration_since(self.fps_last_frame).as_secs_f32();
                self.fps_last_frame = now;

                if dt > 0.0 {
                    const SMOOTHING: f32 = 0.1; // weight of the newest sample.
                    if self.avg_frame_time == 0.0 {
                        self.avg_frame_time = dt;
                    } else {
                        self.avg_frame_time += SMOOTHING * (dt - self.avg_frame_time);
                    }
                }

                // Refresh the HUD text a few times per second, not every frame.
                let refresh = now.duration_since(self.hud_last_update).as_secs_f32() >= 0.25;
                if refresh {
                    self.hud_last_update = now;
                }

                let frame_ms = self.avg_frame_time * 1000.0;
                let fps = if self.avg_frame_time > 0.0 {
                    1.0 / self.avg_frame_time
                } else {
                    0.0
                };

                let State::Running(running) = &mut self.state else {
                    return;
                };
                if refresh {
                    let text = format!("FPS: {fps:.0}  {frame_ms:.2} MS");
                    running.hud.set_text(&running.gpu.queue, &text, [8.0, 8.0]);
                }
                running.gpu.render(&running.world, &running.hud);
            }

            _ => {}
        }
    }
}

impl Gui {
    pub fn run(&mut self) -> Result<(), ()> {
        let event_loop = EventLoop::new().expect("failed to create event loop");
        event_loop.run_app(self).map_err(|_| ())?;
        Ok(())
    }
}

/// Temporary map used until the real world is implemented: a small grid of
/// grass tiles cycling through the three grass top variants.
fn placeholder_map() -> game::Map {
    let width: u16 = 8;
    let height: u16 = 6;
    let tiles = (0..(width as usize * height as usize))
        .map(|i| game::Tile::Grass {
            variant: (i % 3) as u8,
        })
        .collect();
    game::Map {
        width,
        height,
        tiles,
    }
}
