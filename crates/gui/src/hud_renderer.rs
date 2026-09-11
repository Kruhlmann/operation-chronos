use game::constants::{FONT_CHARSET, FONT_GLYPH_SIZE, FONT_MAX_CHARACTERS};
use game::world::Marquee;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct HudUniform {
    // xy = surface size (px), z = glyph count in atlas, w = glyph pixel size.
    params: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GlyphInstance {
    // xy = top-left position (px), z = glyph index, w = unused.
    data: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RectInstance {
    // xy = top-left px, zw = size px.
    rect: [f32; 4],
    fill: [f32; 4],
    border: [f32; 4],
    // x = border thickness (px)
    params: [f32; 4],
}

pub struct HudRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    scale: f32,
    instance_count: u32,

    rect_pipeline: wgpu::RenderPipeline,
    rect_bind_group: wgpu::BindGroup,
    rect_instance_buffer: wgpu::Buffer,
    rect_visible: bool,
}

impl HudRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        surface_size: [f32; 2],
        scale: f32,
    ) -> Self {
        let glyph_count = FONT_CHARSET.len() as u32;
        let (_texture, view, sampler) = build_font_atlas(device, queue, glyph_count);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hud shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(*crate::shaders::HUD)),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hud uniform"),
            size: std::mem::size_of::<HudUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hud instances"),
            size: (std::mem::size_of::<GlyphInstance>() as u64) * FONT_MAX_CHARACTERS as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("hud bind group layout"),
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
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hud bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("hud pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GlyphInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            }],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("hud pipeline"),
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

        let rect_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hud rect shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(*crate::shaders::HUD_RECT)),
        });

        let rect_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("hud rect bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let rect_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hud rect bind group"),
            layout: &rect_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let rect_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("hud rect pipeline layout"),
            bind_group_layouts: &[Some(&rect_bind_group_layout)],
            immediate_size: 0,
        });

        let rect_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hud rect instance"),
            size: std::mem::size_of::<RectInstance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let rect_instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RectInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 32,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 48,
                    shader_location: 3,
                },
            ],
        };

        let rect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("hud rect pipeline"),
            layout: Some(&rect_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &rect_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(rect_instance_layout)],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &rect_shader,
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

        let renderer = Self {
            pipeline,
            bind_group,
            uniform_buffer,
            instance_buffer,
            scale,
            instance_count: 0,
            rect_pipeline,
            rect_bind_group,
            rect_instance_buffer,
            rect_visible: false,
        };
        renderer.upload_uniform(queue, surface_size);
        renderer
    }

    pub fn set_surface_size(&self, queue: &wgpu::Queue, surface_size: [f32; 2]) {
        self.upload_uniform(queue, surface_size);
    }

    fn upload_uniform(&self, queue: &wgpu::Queue, surface_size: [f32; 2]) {
        let uniform = HudUniform {
            params: [
                surface_size[0],
                surface_size[1],
                FONT_CHARSET.len() as f32,
                FONT_GLYPH_SIZE as f32 * self.scale,
            ],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    pub fn set_text(&mut self, queue: &wgpu::Queue, text: &str, origin: [f32; 2]) {
        let advance = FONT_GLYPH_SIZE as f32 * self.scale;
        let mut instances: Vec<GlyphInstance> = Vec::with_capacity(text.len());

        let mut pen_x = origin[0];
        for ch in text.chars() {
            let upper = ch.to_ascii_uppercase() as u8;
            if let Some(index) = FONT_CHARSET.iter().position(|&c| c == upper) {
                instances.push(GlyphInstance {
                    data: [pen_x, origin[1], index as f32, 0.0],
                });
            }
            pen_x += advance;

            if instances.len() >= FONT_MAX_CHARACTERS {
                break;
            }
        }

        self.instance_count = instances.len() as u32;
        if !instances.is_empty() {
            queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
        }
    }

    pub fn set_marquee(&mut self, queue: &wgpu::Queue, marquee: Option<&Marquee>) {
        let Some(m) = marquee else {
            self.rect_visible = false;
            return;
        };
        let min = m.min();
        let max = m.max();
        let size = [max[0] - min[0], max[1] - min[1]];
        if size[0] <= 0.0 || size[1] <= 0.0 {
            self.rect_visible = false;
            return;
        }
        // Teal: rgb ~ (0.10, 0.75, 0.75)
        let inst = RectInstance {
            rect: [min[0], min[1], size[0], size[1]],
            fill: [0.10, 0.75, 0.75, 0.08],
            border: [0.10, 0.75, 0.75, 0.55],
            params: [1.5, 0.0, 0.0, 0.0],
        };
        queue.write_buffer(&self.rect_instance_buffer, 0, bytemuck::bytes_of(&inst));
        self.rect_visible = true;
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        if self.rect_visible {
            render_pass.set_pipeline(&self.rect_pipeline);
            render_pass.set_bind_group(0, &self.rect_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.rect_instance_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }
        if self.instance_count == 0 {
            return;
        }
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        render_pass.draw(0..6, 0..self.instance_count);
    }
}

fn build_font_atlas(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    glyph_count: u32,
) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
    let width = glyph_count * FONT_GLYPH_SIZE;
    let height = FONT_GLYPH_SIZE;
    let mut pixels = vec![0u8; (width * height) as usize];

    for (glyph_index, rows) in FONT.iter().enumerate() {
        let x0 = glyph_index as u32 * FONT_GLYPH_SIZE;
        for (row, bits) in rows.iter().enumerate() {
            for col in 0..FONT_GLYPH_SIZE {
                let on = (bits >> (FONT_GLYPH_SIZE - 1 - col)) & 1 == 1;
                if on {
                    let x = x0 + col;
                    let y = row as u32;
                    pixels[(y * width + x) as usize] = 255;
                }
            }
        }
    }

    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("hud font atlas"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width),
            rows_per_image: Some(height),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("hud font sampler"),
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });

    (texture, view, sampler)
}

#[rustfmt::skip]
const FONT: &[[u8; 8]] = &[
    // ' '
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    // '.'
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00],
    // ':'
    [0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00, 0x00],
    // '0'
    [0x3C, 0x66, 0x6E, 0x76, 0x66, 0x66, 0x3C, 0x00],
    // '1'
    [0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00],
    // '2'
    [0x3C, 0x66, 0x06, 0x0C, 0x30, 0x60, 0x7E, 0x00],
    // '3'
    [0x3C, 0x66, 0x06, 0x1C, 0x06, 0x66, 0x3C, 0x00],
    // '4'
    [0x0C, 0x1C, 0x3C, 0x6C, 0x7E, 0x0C, 0x0C, 0x00],
    // '5'
    [0x7E, 0x60, 0x7C, 0x06, 0x06, 0x66, 0x3C, 0x00],
    // '6'
    [0x1C, 0x30, 0x60, 0x7C, 0x66, 0x66, 0x3C, 0x00],
    // '7'
    [0x7E, 0x06, 0x0C, 0x18, 0x30, 0x30, 0x30, 0x00],
    // '8'
    [0x3C, 0x66, 0x66, 0x3C, 0x66, 0x66, 0x3C, 0x00],
    // '9'
    [0x3C, 0x66, 0x66, 0x3E, 0x06, 0x0C, 0x38, 0x00],
    // 'A'
    [0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x00],
    // 'B'
    [0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x7C, 0x00],
    // 'C'
    [0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00],
    // 'D'
    [0x78, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0x78, 0x00],
    // 'E'
    [0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x7E, 0x00],
    // 'F'
    [0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x60, 0x00],
    // 'G'
    [0x3C, 0x66, 0x60, 0x6E, 0x66, 0x66, 0x3C, 0x00],
    // 'H'
    [0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00],
    // 'I'
    [0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00],
    // 'J'
    [0x1E, 0x0C, 0x0C, 0x0C, 0x0C, 0x6C, 0x38, 0x00],
    // 'K'
    [0x66, 0x6C, 0x78, 0x70, 0x78, 0x6C, 0x66, 0x00],
    // 'L'
    [0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x7E, 0x00],
    // 'M'
    [0x63, 0x77, 0x7F, 0x6B, 0x63, 0x63, 0x63, 0x00],
    // 'N'
    [0x66, 0x76, 0x7E, 0x7E, 0x6E, 0x66, 0x66, 0x00],
    // 'O'
    [0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00],
    // 'P'
    [0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0x00],
    // 'Q'
    [0x3C, 0x66, 0x66, 0x66, 0x6E, 0x6C, 0x36, 0x00],
    // 'R'
    [0x7C, 0x66, 0x66, 0x7C, 0x78, 0x6C, 0x66, 0x00],
    // 'S'
    [0x3C, 0x66, 0x60, 0x3C, 0x06, 0x66, 0x3C, 0x00],
    // 'T'
    [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00],
    // 'U'
    [0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00],
    // 'V'
    [0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00],
    // 'W'
    [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00],
    // 'X'
    [0x66, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x66, 0x00],
    // 'Y'
    [0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x18, 0x00],
    // 'Z'
    [0x7E, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x7E, 0x00],
];
