#![allow(clippy::disallowed_types)]

use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Asset {
    Png(String),
    Oog(String),
}

impl Asset {
    pub fn path(&self) -> &str {
        match self {
            Asset::Png(path) => path,
            Asset::Oog(path) => path,
        }
    }
}

pub struct AssetLibrary {
    pub asset_directory: String,
    pub assets: HashSet<Asset>,
}

impl AssetLibrary {
    pub fn load_asset_directory(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut assets = HashSet::new();
        let dir = std::path::Path::new(path);
        Self::populate_assets(dir, &mut assets)?;
        tracing::info!("found {} valid assets in {path:?}", assets.len());
        Ok(Self {
            assets,
            asset_directory: path.to_string(),
        })
    }

    fn populate_assets(
        dir: &std::path::Path,
        assets: &mut HashSet<Asset>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                Self::populate_assets(&path, assets)?;
            } else {
                match path.extension() {
                    Some(ext) if ext == "png" => {
                        assets.insert(Asset::Png(path.to_string_lossy().to_string()));
                    }
                    Some(ext) if ext == "oog" => {
                        assets.insert(Asset::Oog(path.to_string_lossy().to_string()));
                    }
                    _ => {}
                };
            }
        }
        Ok(())
    }

    pub fn get_asset(&self, path: &str) -> Option<&Asset> {
        self.assets
            .iter()
            .find(|asset| asset.path() == format!("{}/{path}", self.asset_directory))
    }
}

pub struct LoadedTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

pub struct GpuAssets {
    pub textures: HashMap<String, LoadedTexture>,
}

impl GpuAssets {
    pub fn load(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        library: &AssetLibrary,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut textures = HashMap::new();

        for asset in &library.assets {
            let Asset::Png(path) = asset else {
                continue;
            };
            let loaded = Self::load_png(device, queue, path)?;
            textures.insert(path.clone(), loaded);
        }

        Ok(Self { textures })
    }

    pub fn get(&self, path: &str) -> Option<&LoadedTexture> {
        self.textures.get(path)
    }

    fn load_png(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &str,
    ) -> Result<LoadedTexture, Box<dyn std::error::Error>> {
        let image = image::open(path)?.to_rgba8();
        let (width, height) = image.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(path),
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
            label: Some(path),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Ok(LoadedTexture {
            texture,
            view,
            sampler,
        })
    }
}
