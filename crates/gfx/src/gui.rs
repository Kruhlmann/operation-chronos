use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::{
    AssetLibrary, Gpu, GpuAssets, HudRenderer, Input, InputEventSideEffect, LineRenderer,
    SelectionEntry, SheetId, UnitScene, WorldRenderer,
};
use data::constants::{
    ASSET_DIRECTORY, FRAME_TIME, SPRITE_SHEET_COLUMNS, SPRITE_SHEET_PATH, SPRITE_SHEET_ROWS,
    SPRITE_SIZE_PIXELS, TICK_TIME,
};
use data::geometry::{Disc, Footprint, Position};

use gameplay::{ClientView, Sim};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};
use world::Map;
use world::entity::{Health, Name, UnitKind, UnitMoveInstructions};

pub struct RunningState {
    pub window: Arc<Window>,
    pub gpu: Gpu,
    pub assets: GpuAssets,
    pub renderer: WorldRenderer,
    pub units: UnitScene,
    pub selection: LineRenderer,
    pub lines: LineRenderer,
    pub debug_lines: LineRenderer,
    pub debug_overlay: bool,
    pub sim: Sim,
    pub view: ClientView,
    pub input: Input,
    pub hud: HudRenderer,
    last_tick: Instant,
    tick_accumulator: Duration,
}

pub enum State {
    Uninitialized,
    Running(Box<RunningState>),
}

pub struct Gui {
    pub state: State,
    pub pending: Option<(Sim, ClientView)>,
    pub library: AssetLibrary,
    pub next_frame_scheduled: Instant,
    pub initial_logical_size: [u32; 2],
    fps_last_frame: Instant,
    avg_frame_time: f32,
    hud_last_update: Instant,
}

impl Gui {
    pub fn new(library: AssetLibrary, sim: Sim, view: ClientView) -> Self {
        Self {
            pending: Some((sim, view)),
            library,
            state: State::Uninitialized,
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

        let (sim, view) = self.pending.take().expect("sim already moved");

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

        let assets = GpuAssets::load(&gpu.device, &gpu.queue, &self.library)
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

        let mut renderer = WorldRenderer::new(
            &gpu.device,
            &gpu.queue,
            gpu.config.format,
            sprite_sheet,
            SPRITE_SHEET_COLUMNS,
            SPRITE_SHEET_ROWS,
            SPRITE_SIZE_PIXELS as f32,
            &view.camera,
        );
        renderer.set_map(&gpu.queue, &sim.map);

        let units = UnitScene::new(
            &gpu.device,
            &gpu.queue,
            gpu.config.format,
            &assets,
            &view.camera,
            SheetId::ALL,
        );

        let selection = LineRenderer::new(&gpu.device, &gpu.queue, gpu.config.format, &view.camera);
        let lines = LineRenderer::new(&gpu.device, &gpu.queue, gpu.config.format, &view.camera);
        let debug_lines =
            LineRenderer::new(&gpu.device, &gpu.queue, gpu.config.format, &view.camera);

        self.state = State::Running(Box::new(RunningState {
            window,
            gpu,
            assets,
            renderer,
            units,
            selection,
            lines,
            debug_lines,
            debug_overlay: false,
            sim,
            view,
            input: Input::default(),
            hud,
            last_tick: Instant::now(),
            tick_accumulator: Duration::ZERO,
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
        let Self {
            state,
            fps_last_frame,
            avg_frame_time,
            hud_last_update,
            ..
        } = self;
        let State::Running(running) = state else {
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
                    running.view.resize(viewport);
                    running
                        .renderer
                        .set_camera(&running.gpu.queue, &running.view.camera);
                    running
                        .units
                        .set_camera(&running.gpu.queue, &running.view.camera);
                }
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now.duration_since(*fps_last_frame).as_secs_f32();
                *fps_last_frame = now;

                if dt > 0.0 {
                    const SMOOTHING: f32 = 0.1;
                    if *avg_frame_time == 0.0 {
                        *avg_frame_time = dt;
                    } else {
                        *avg_frame_time += SMOOTHING * (dt - *avg_frame_time);
                    }
                }
                let refresh = now.duration_since(*hud_last_update).as_secs_f32() >= 0.25;
                if refresh {
                    *hud_last_update = now;
                }

                let frame_ms = *avg_frame_time * 1000.0;
                let fps = if *avg_frame_time > 0.0 {
                    1.0 / *avg_frame_time
                } else {
                    0.0
                };

                if refresh {
                    let text = format!("FPS: {fps:.0}  {frame_ms:.2} MS");
                    running.hud.set_text(&running.gpu.queue, &text, [8.0, 8.0]);
                }
                running
                    .hud
                    .set_marquee(&running.gpu.queue, running.view.selection.marquee.as_ref());

                let selection_entries: Vec<SelectionEntry> = running
                    .view
                    .selection
                    .selected
                    .iter()
                    .filter_map(|&e| {
                        running
                            .sim
                            .ecs
                            .query_one::<(&UnitKind, &Name, &Health)>(e)
                            .ok()?
                            .get()
                            .map(|(k, n, h)| SelectionEntry {
                                kind: *k,
                                name: n.0,
                                health_fraction: h.fraction(),
                            })
                    })
                    .collect();
                let viewport = [
                    running.gpu.config.width as f32,
                    running.gpu.config.height as f32,
                ];
                running
                    .hud
                    .set_selection_panel(&running.gpu.queue, viewport, &selection_entries);

                let tick_dt = now.duration_since(running.last_tick);
                running.last_tick = now;
                running.tick_accumulator =
                    (running.tick_accumulator + tick_dt).min(Duration::from_millis(250));
                while running.tick_accumulator >= TICK_TIME {
                    running.sim.tick(TICK_TIME);
                    running.tick_accumulator -= TICK_TIME;
                }

                running.units.refresh(&running.gpu.queue, &running.sim.ecs);
                refresh_selection_markers(
                    &mut running.selection,
                    &running.gpu.queue,
                    &running.sim.ecs,
                    &running.view.selection.selected,
                );
                if running.debug_overlay {
                    refresh_move_lines(&mut running.lines, &running.gpu.queue, &running.sim.ecs);
                    refresh_debug_overlay(
                        &mut running.debug_lines,
                        &running.gpu.queue,
                        &running.sim,
                    );
                } else {
                    running.lines.set_segments(&running.gpu.queue, &[]);
                    running.debug_lines.set_segments(&running.gpu.queue, &[]);
                }
                running
                    .selection
                    .set_camera(&running.gpu.queue, &running.view.camera);
                running
                    .lines
                    .set_camera(&running.gpu.queue, &running.view.camera);
                running
                    .debug_lines
                    .set_camera(&running.gpu.queue, &running.view.camera);
                running
                    .units
                    .set_camera(&running.gpu.queue, &running.view.camera);
                running
                    .renderer
                    .set_camera(&running.gpu.queue, &running.view.camera);
                running.gpu.render(
                    &running.renderer,
                    &running.selection,
                    &running.units,
                    &running.lines,
                    if running.debug_overlay {
                        Some(&running.debug_lines)
                    } else {
                        None
                    },
                    &running.hud,
                );
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::F3),
                        state: ElementState::Pressed,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                running.debug_overlay = !running.debug_overlay;
            }

            other => match running.input.handle(&other, &mut running.view) {
                Some(InputEventSideEffect::UpdateCamera) => {
                    running
                        .renderer
                        .set_camera(&running.gpu.queue, &running.view.camera);
                    running
                        .units
                        .set_camera(&running.gpu.queue, &running.view.camera);
                    running
                        .selection
                        .set_camera(&running.gpu.queue, &running.view.camera);
                    running
                        .lines
                        .set_camera(&running.gpu.queue, &running.view.camera);
                }
                Some(InputEventSideEffect::CommitMarquee) => {
                    running.view.commit_marquee(&running.sim);
                }
                Some(InputEventSideEffect::ClickLeft(p))
                    if !is_in_selection_panel(p, &running.gpu) =>
                {
                    running.view.click_select(&running.sim, p)
                }
                Some(InputEventSideEffect::ClickRight(p))
                    if !is_in_selection_panel(p, &running.gpu) =>
                {
                    running.view.click_order(&mut running.sim, p)
                }
                Some(InputEventSideEffect::ClickLeft(_))
                | Some(InputEventSideEffect::ClickRight(_)) => {}
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

const SELECTION_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 0.9];

fn is_in_selection_panel(cursor: [f32; 2], gpu: &Gpu) -> bool {
    let panel_top = crate::renderer::hud::selection_panel_top(gpu.config.height as f32);
    cursor[1] >= panel_top
}

fn push_disc_outline(
    segs: &mut Vec<(glam::Vec2, glam::Vec2, [f32; 4])>,
    disc: Disc,
    color: [f32; 4],
) {
    const SEGMENTS: usize = 24;
    let mut points: Vec<glam::Vec2> = disc.outline_render(SEGMENTS).collect();
    points.push(points[0]);
    for w in points.windows(2) {
        segs.push((w[0], w[1], color));
    }
}

fn refresh_selection_markers(
    renderer: &mut LineRenderer,
    queue: &wgpu::Queue,
    ecs: &hecs::World,
    selected: &[hecs::Entity],
) {
    let mut segs: Vec<(glam::Vec2, glam::Vec2, [f32; 4])> = Vec::new();
    for &e in selected {
        let Ok(mut q) = ecs.query_one::<(&Position, &Footprint)>(e) else {
            continue;
        };
        if let Some((pos, fp)) = q.get() {
            push_disc_outline(&mut segs, fp.disc_at(*pos), SELECTION_COLOR);
        }
    }
    renderer.set_segments(queue, &segs);
}

fn refresh_move_lines(renderer: &mut LineRenderer, queue: &wgpu::Queue, ecs: &hecs::World) {
    let mut segments: Vec<(glam::Vec2, glam::Vec2, [f32; 4])> = Vec::new();
    let color = [0.1, 0.95, 0.2, 1.0];
    for (_e, (pos, order)) in ecs.query::<(&Position, &UnitMoveInstructions)>().iter() {
        let mut prev = pos.to_render();
        for &wp in &order.waypoints {
            let next = wp.to_render();
            segments.push((prev, next, color));
            prev = next;
        }
    }
    renderer.set_segments(queue, &segments);
}

const DEBUG_TILE_COLOR: [f32; 4] = [1.0, 0.2, 0.2, 0.9];
const DEBUG_UNIT_COLOR: [f32; 4] = [0.2, 1.0, 0.4, 0.9];

fn refresh_debug_overlay(renderer: &mut LineRenderer, queue: &wgpu::Queue, sim: &Sim) {
    use data::constants::{ISOMETRIC_TILE_HALF_HEIGHT, ISOMETRIC_TILE_HALF_WIDTH};
    let mut segs: Vec<(glam::Vec2, glam::Vec2, [f32; 4])> = Vec::new();

    let diamond = |center: glam::Vec2, tiles: f32, color: [f32; 4], out: &mut Vec<_>| {
        let hw = tiles * ISOMETRIC_TILE_HALF_WIDTH;
        let hh = tiles * ISOMETRIC_TILE_HALF_HEIGHT;
        let top = center + glam::Vec2::new(0.0, -hh);
        let right = center + glam::Vec2::new(hw, 0.0);
        let bottom = center + glam::Vec2::new(0.0, hh);
        let left = center + glam::Vec2::new(-hw, 0.0);
        out.push((top, right, color));
        out.push((right, bottom, color));
        out.push((bottom, left, color));
        out.push((left, top, color));
    };

    for ty in 0..sim.map.height as i32 {
        for tx in 0..sim.map.width as i32 {
            if !sim.map.is_passable(tx, ty) {
                let [ax, ay] = Map::tile_to_world(tx as u16, ty as u16);
                let center = glam::Vec2::new(ax, ay + ISOMETRIC_TILE_HALF_HEIGHT);
                diamond(center, 1.0, DEBUG_TILE_COLOR, &mut segs);
            }
        }
    }

    for (_e, (pos, fp)) in sim.ecs.query::<(&Position, &Footprint)>().iter() {
        push_disc_outline(&mut segs, fp.disc_at(*pos), DEBUG_UNIT_COLOR);
    }

    renderer.set_segments(queue, &segs);
}
