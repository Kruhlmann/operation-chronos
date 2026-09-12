use std::collections::HashMap;

use glam::Vec2;
use world::entity::UnitKind;
use world::geometry::{Facing, Position};
use world::{Camera, GpuAssets};

use crate::UnitRenderer;
use crate::appearance::{SheetId, SpritePart, SpriteRef};

pub struct UnitScene {
    renderers: HashMap<SheetId, UnitRenderer>,
    order: Vec<SheetId>,
    appearances: HashMap<UnitKind, Vec<SpritePart>>,
}

impl UnitScene {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        assets: &GpuAssets,
        camera: &Camera,
        sheets: &[SheetId],
    ) -> Self {
        let mut renderers = HashMap::new();
        let mut order = Vec::new();
        for &sheet in sheets {
            let path = sheet.asset_path();
            let tex = assets
                .get(path)
                .unwrap_or_else(|| panic!("sprite sheet not loaded: {path}"));
            let (fw, fh) = sheet.frame_size();
            let r = UnitRenderer::new(device, queue, format, tex, fw, fh, camera);
            renderers.insert(sheet, r);
            order.push(sheet);
        }
        Self {
            renderers,
            order,
            appearances: default_appearances(),
        }
    }

    pub fn set_camera(&self, queue: &wgpu::Queue, camera: &Camera) {
        for r in self.renderers.values() {
            r.set_camera(queue, camera);
        }
    }

    pub fn refresh(&mut self, queue: &wgpu::Queue, ecs: &hecs::World) {
        let mut batches: HashMap<SheetId, Vec<(Vec2, u32)>> = HashMap::new();
        for &sheet in &self.order {
            batches.insert(sheet, Vec::new());
        }

        let mut q = ecs.query::<(&Position, &Facing, &UnitKind)>();
        for (_e, (pos, facing, kind)) in q.iter() {
            let Some(parts) = self.appearances.get(kind) else {
                continue;
            };
            for part in parts {
                let face_val = part.facing_override.map(Facing).unwrap_or(*facing);
                let frame = part.sprite.frame_for(face_val);
                let world_pos = pos.0 + part.offset;
                if let Some(bin) = batches.get_mut(&part.sprite.sheet) {
                    bin.push((world_pos, frame));
                }
            }
        }

        for (sheet, items) in batches {
            if let Some(r) = self.renderers.get_mut(&sheet) {
                r.set_instances(queue, &items);
            }
        }
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        for sheet in &self.order {
            if let Some(r) = self.renderers.get(sheet) {
                r.draw(render_pass);
            }
        }
    }
}

fn default_appearances() -> HashMap<UnitKind, Vec<SpritePart>> {
    let mut m = HashMap::new();
    m.insert(
        UnitKind::Tank,
        vec![
            SpritePart {
                sprite: SpriteRef {
                    sheet: SheetId::TankBody,
                    frame_base: 0,
                    frame_count: 8,
                },
                offset: Vec2::ZERO,
                facing_override: None,
            },
            SpritePart {
                sprite: SpriteRef {
                    sheet: SheetId::TankTurret,
                    frame_base: 0,
                    frame_count: 8,
                },
                offset: Vec2::ZERO,
                facing_override: None,
            },
        ],
    );
    m
}
