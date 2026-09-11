use std::sync::Arc;
use std::time::Instant;

use crate::{Gpu, HudRenderer, Input, InputEventSideEffect, WorldRenderer};
use world::constants::{
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
    pub assets: world::GpuAssets,
    pub renderer: WorldRenderer,
    pub state: world::World,
    pub input: Input,
    pub hud: HudRenderer,
}

pub enum State {
    Uninitialized,
    Running(Box<Running>),
}

pub struct Gui {
    pub state: State,
    pub library: world::AssetLibrary,
    pub next_frame_scheduled: Instant,
    pub initial_logical_size: [u32; 2],
    fps_last_frame: Instant,
    avg_frame_time: f32,
    hud_last_update: Instant,
}

impl Gui {
    pub fn new(library: world::AssetLibrary) -> Self {
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

        let assets = world::GpuAssets::load(&gpu.device, &gpu.queue, &self.library)
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

        let world_state = world::World::placeholder(surface_size);

        let mut renderer = WorldRenderer::new(
            &gpu.device,
            &gpu.queue,
            gpu.config.format,
            sprite_sheet,
            SPRITE_SHEET_COLUMNS,
            SPRITE_SHEET_ROWS,
            SPRITE_SIZE_PIXELS as f32,
            &world_state.camera,
        );
        renderer.set_map(&gpu.queue, &world_state.map);

        self.state = State::Running(Box::new(Running {
            window,
            gpu,
            assets,
            renderer,
            state: world_state,
            input: Input::default(),
            hud,
        }));
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
        let State::Running(running) = &mut self.state else {
            return;
        };
        if running.window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                running.gpu.resize(size.width, size.height);
                if size.height > 0 {
                    let viewport = [size.width as f32, size.height as f32];
                    running.hud.set_surface_size(&running.gpu.queue, viewport);
                    running.state.resize(viewport);
                    running
                        .renderer
                        .set_camera(&running.gpu.queue, &running.state.camera);
                }
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now.duration_since(self.fps_last_frame).as_secs_f32();
                self.fps_last_frame = now;

                if dt > 0.0 {
                    const SMOOTHING: f32 = 0.1;
                    if self.avg_frame_time == 0.0 {
                        self.avg_frame_time = dt;
                    } else {
                        self.avg_frame_time += SMOOTHING * (dt - self.avg_frame_time);
                    }
                }
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

                if refresh {
                    let text = format!("FPS: {fps:.0}  {frame_ms:.2} MS");
                    running.hud.set_text(&running.gpu.queue, &text, [8.0, 8.0]);
                }
                running
                    .hud
                    .set_marquee(&running.gpu.queue, running.state.selection.marquee.as_ref());
                running.gpu.render(&running.renderer, &running.hud);
            }

            other => match running.input.handle(&other, &mut running.state) {
                Some(InputEventSideEffect::UpdateCamera) => running
                    .renderer
                    .set_camera(&running.gpu.queue, &running.state.camera),
                Some(InputEventSideEffect::ClickLeft(p))
                | Some(InputEventSideEffect::ClickRight(p)) => eprintln!("click {p:?}"),
                None => {}
            },
        }
    }
}

impl Gui {
    #[allow(clippy::result_unit_err)]
    pub fn run(&mut self) -> Result<(), ()> {
        let event_loop = EventLoop::new().expect("failed to create event loop");
        event_loop.run_app(self).map_err(|_| ())?;
        Ok(())
    }
}
