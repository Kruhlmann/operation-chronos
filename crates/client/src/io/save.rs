use std::{
    io::{Read, Write},
    path::Path,
};

use data::io::{CompressedBytes, SaveHeader, Saveable};

pub struct SaveableFormatLoader;

impl SaveableFormatLoader {
    pub fn write<T: Saveable>(value: &T, directory: &str, file_name: &str) -> std::io::Result<()> {
        let path = Path::new(directory)
            .join(file_name)
            .with_extension(T::EXTENSION);
        tracing::debug!(kind = T::KIND, ?path, "writing saveable");

        if path.exists() {
            std::fs::remove_file(&path)
                .inspect_err(|error| tracing::error!(?error, ?path, "remove file"))?;
        }

        let compressed = value.encode()?;
        let mut file = std::fs::File::create(&path)
            .inspect_err(|error| tracing::error!(?error, ?path, "create file"))?;

        SaveHeader::current::<T>().write(&mut file)?;
        file.write_all(&compressed.0)?;

        tracing::info!(kind = T::KIND, ?path, "wrote saveable");
        Ok(())
    }

    pub fn read<T: Saveable>(file_path: &str) -> std::io::Result<T> {
        let path = Path::new(file_path);
        tracing::debug!(kind = T::KIND, ?path, "reading saveable");

        let mut file = std::fs::File::open(path)
            .inspect_err(|error| tracing::error!(?error, ?path, "open file"))?;

        let header = SaveHeader::read(&mut file)?;
        header.verify_header::<T>()?;

        let mut body = Vec::new();
        file.read_to_end(&mut body)?;
        let value = T::decode(CompressedBytes(body))?;

        tracing::info!(kind = T::KIND, ?path, "read saveable");
        Ok(value)
    }
}
