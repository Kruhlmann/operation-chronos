use serde::{Serialize, de::DeserializeOwned};

use crate::{
    constants::{BINCODE_CONFIG, GAME_VERSION_BINARY},
    io::CompressedBytes,
};

pub trait Saveable: Sized + Serialize + DeserializeOwned {
    const MAGIC: [u8; 8];
    const EXTENSION: &'static str;
    const KIND: &'static str;

    fn encode(&self) -> std::io::Result<CompressedBytes> {
        let raw = bincode::serde::encode_to_vec(self, *BINCODE_CONFIG)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        raw.try_into()
    }

    fn decode(bytes: CompressedBytes) -> std::io::Result<Self> {
        let raw: Vec<u8> = bytes.try_into()?;
        let (value, _) = bincode::serde::decode_from_slice(&raw, *BINCODE_CONFIG)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SaveHeader {
    pub magic: [u8; 8],
    pub version: [u8; 3],
}

impl SaveHeader {
    pub const SIZE: usize = 11;

    pub fn current<T: Saveable>() -> Self {
        Self {
            magic: T::MAGIC,
            version: *GAME_VERSION_BINARY,
        }
    }

    pub fn read<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;
        let mut version = [0u8; 3];
        reader.read_exact(&mut version)?;
        Ok(Self { magic, version })
    }

    pub fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.magic)?;
        writer.write_all(&self.version)
    }

    pub fn verify_header<T: Saveable>(&self) -> std::io::Result<()> {
        if self.magic != T::MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "expected {} file (magic {:x?}) but found magic {:x?}",
                    T::KIND,
                    T::MAGIC,
                    self.magic
                ),
            ));
        }
        if self.version != *GAME_VERSION_BINARY {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "version mismatch - found {:?} expected {:?}",
                    self.version, *GAME_VERSION_BINARY
                ),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Dummy {
        n: u32,
        s: String,
    }

    impl Saveable for Dummy {
        const MAGIC: [u8; 8] = [0xaa, 0xbb, 0xcc, 0xdd, 0x00, 0x01, 0x02, 0x03];
        const EXTENSION: &'static str = "dummy";
        const KIND: &'static str = "dummy";
    }

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Other(u8);

    impl Saveable for Other {
        const MAGIC: [u8; 8] = [0x11; 8];
        const EXTENSION: &'static str = "other";
        const KIND: &'static str = "other";
    }

    #[test]
    fn header_roundtrip() {
        let mut buf = Vec::new();
        SaveHeader::current::<Dummy>().write(&mut buf).unwrap();
        assert_eq!(buf.len(), SaveHeader::SIZE);

        let mut cursor = std::io::Cursor::new(buf);
        let header = SaveHeader::read(&mut cursor).unwrap();
        header.verify_header::<Dummy>().unwrap();
    }

    #[test]
    fn header_verify_rejects_wrong_type() {
        let header = SaveHeader::current::<Dummy>();
        let err = header.verify_header::<Other>().unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn encode_decode_roundtrip() {
        let value = Dummy {
            n: 42,
            s: "hello".to_string(),
        };
        let compressed = value.encode().unwrap();
        let decoded = Dummy::decode(compressed).unwrap();
        assert_eq!(value, decoded);
    }
}
