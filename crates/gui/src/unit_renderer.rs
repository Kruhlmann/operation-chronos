use world::{Camera, LoadedTexture};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_projection: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SheetUniform {
    // x = sheet width px, y = sheet height px, z = frame width px, w = frame height px.
    params: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct UnitInstance {
    // xy = world-pixel top-left, z = frame index, w = unused.
    data: [f32; 4],
}

const MAX_UNITS: u64 = 1024;

pub struct UnitRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_count: u32,
    frame_width: f32,
    frame_height: f32,
    total_frames: u32,
}

impl UnitRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        sheet: &LoadedTexture,
        frame_width: f32,
        frame_height: f32,
        camera: &Camera,
    ) -> Self {
        let sheet_size = sheet.texture.size();
        let sheet_w = sheet_size.width as f32;
        let sheet_h = sheet_size.height as f32;
        let total_frames = (sheet_w / frame_width) as u32;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("unit shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(*crate::shaders::UNIT)),
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("unit camera uniform"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sheet_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("unit sheet uniform"),
            size: std::mem::size_of::<SheetUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("unit instances"),
            size: std::mem::size_of::<UnitInstance>() as u64 * MAX_UNITS,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("unit bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("unit bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&sheet.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sheet.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: sheet_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("unit pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UnitInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            }],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("unit pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(instance_layout)],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        queue.write_buffer(
            &sheet_buffer,
            0,
            bytemuck::bytes_of(&SheetUniform {
                params: [sheet_w, sheet_h, frame_width, frame_height],
            }),
        );

        let renderer = Self {
            pipeline,
            bind_group,
            camera_buffer,
            instance_buffer,
            instance_count: 0,
            frame_width,
            frame_height,
            total_frames,
        };
        renderer.set_camera(queue, camera);
        renderer
    }

    pub fn set_camera(&self, queue: &wgpu::Queue, camera: &Camera) {
        let uniform = CameraUniform {
            view_projection: camera.view_projection(),
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    pub fn total_frames(&self) -> u32 {
        self.total_frames
    }

    /// Upload a single unit at world position `pos` (center) using `frame`.
    pub fn set_single(&mut self, queue: &wgpu::Queue, pos: [f32; 2], frame: u32) {
        let anchor = [
            pos[0] - self.frame_width * 0.5,
            pos[1] - self.frame_height * 0.5,
        ];
        let inst = UnitInstance {
            data: [anchor[0], anchor[1], frame as f32, 0.0],
        };
        queue.write_buffer(&self.instance_buffer, 0, bytemuck::bytes_of(&inst));
        self.instance_count = 1;
    }

    /// Upload N unit instances. Each `(center, frame)` becomes one draw.
    pub fn set_instances(&mut self, queue: &wgpu::Queue, items: &[(glam::Vec2, u32)]) {
        let hw = self.frame_width * 0.5;
        let hh = self.frame_height * 0.5;
        let capped = items.len().min(MAX_UNITS as usize);
        let mut buf: Vec<UnitInstance> = Vec::with_capacity(capped);
        for &(pos, frame) in items.iter().take(capped) {
            buf.push(UnitInstance {
                data: [pos.x - hw, pos.y - hh, frame as f32, 0.0],
            });
        }
        self.instance_count = buf.len() as u32;
        if !buf.is_empty() {
            queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&buf));
        }
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        if self.instance_count == 0 {
            return;
        }
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        render_pass.draw(0..6, 0..self.instance_count);
    }
}
