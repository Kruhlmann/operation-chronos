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
                    p => {
                        tracing::warn!(
                            "Warning: Unsupported asset type {:?} for file {}",
                            p,
                            path.to_string_lossy()
                        );
                    }
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
