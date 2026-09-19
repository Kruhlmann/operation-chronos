use crate::Asset;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SheetId {
    TankBody,
    TankTurret,
}

impl SheetId {
    pub const ALL: &'static [SheetId] = &[SheetId::TankBody, SheetId::TankTurret];

    pub fn asset_path(&self) -> &'static str {
        match self {
            SheetId::TankBody => "res/unit/double-barrel-tank/body.png",
            SheetId::TankTurret => "res/unit/double-barrel-tank/turret.png",
        }
    }

    pub fn frame_size(&self) -> (f32, f32) {
        match self {
            SheetId::TankBody => (70.0, 48.0),
            SheetId::TankTurret => (62.0, 42.0),
        }
    }

    pub fn facings(&self) -> u32 {
        match self {
            SheetId::TankBody => 32,
            SheetId::TankTurret => 32,
        }
    }
}

pub struct SpriteSheet<T, V, S> {
    pub columns: u16,
    pub rows: u16,
    pub sprite_width: u16,
    pub sprite_height: u16,

    pub texture: T,
    pub view: V,
    pub sampler: S,
}

impl SpriteSheet<(), (), ()> {
    pub fn new(rows: u16, columns: u16, sprite_width: u16, sprite_height: u16) -> Self {
        Self {
            rows,
            columns,
            sprite_width,
            sprite_height,
            texture: (),
            view: (),
            sampler: (),
        }
    }

    pub fn load_png(
        self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        asset: Asset,
    ) -> Result<
        SpriteSheet<wgpu::Texture, wgpu::TextureView, wgpu::Sampler>,
        Box<dyn std::error::Error>,
    > {
        let image = match asset {
            Asset::Png(path) => image::open(path)?.to_rgba8(),
            _ => return Err(format!("Asset {asset:?} is not a PNG image").into()),
        };
        let (width, height) = image.dimensions();

        let expected_width = self.columns as u32 * self.sprite_width as u32;
        let expected_height = self.rows as u32 * self.sprite_height as u32;
        if width != expected_width || height != expected_height {
            return Err(format!(
                "Image dimensions do not match sprite sheet dimensions: expected {}x{}, got {}x{}",
                expected_width, expected_height, width, height,
            )
            .into());
        }

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sprite sheet"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
            image.as_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite sheet sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Ok(SpriteSheet {
            rows: self.rows,
            columns: self.columns,
            sprite_width: self.sprite_width,
            sprite_height: self.sprite_height,
            texture,
            view,
            sampler,
        })
    }
}
