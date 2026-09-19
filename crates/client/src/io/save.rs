use std::{
    io::{Read, Write},
    path::Path,
};

use data::{constants::GAME_VERSION_BINARY, io::SaveableFormat};

pub struct SaveableFormatLoader;

impl SaveableFormatLoader {
    pub fn write_saveable(
        target: SaveableFormat,
        directory: &str,
        file_name: &str,
    ) -> std::io::Result<()> {
        tracing::debug!(%target, %file_name, %directory, "writing savable format");
        let path = Path::new(directory)
            .join(file_name)
            .with_extension(target.file_extension());
        if path.exists() {
            std::fs::remove_file(&path)
                .inspect_err(|error| tracing::error!(?error, ?path, "remove file"))?;
        }
        let mut file = std::fs::File::create(&path)
            .inspect_err(|error| tracing::error!(?error, ?path, "create file"))?;
        file.write_all(&target.header_bytes())?;
        file.write_all(&(*GAME_VERSION_BINARY))?;
        file.write_all(&target.body_bytes())?;
        tracing::info!(%file_name, %directory, "wrote savable format");
        Ok(())
    }

    pub fn read_saveable(file_path: &str) -> std::io::Result<SaveableFormat> {
        tracing::debug!(%file_path, "reading savable format");
        let path = Path::new(&file_path);
        let mut file = std::fs::File::open(path)
            .inspect_err(|error| tracing::error!(?error, ?path, "read file"))?;
        let mut header = [0u8; 8];
        file.read_exact(&mut header)?;
        let mut version = [0u8; 3];
        file.read_exact(&mut version)?;
        let mut body = Vec::new();
        file.read_to_end(&mut body)?;

        let target = SaveableFormat::from_read_bytes(&header, &version, &body)?;

        tracing::info!(%file_path, %target, "read savable format");

        Ok(target)
    }
}
