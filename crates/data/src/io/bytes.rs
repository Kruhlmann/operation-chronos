use std::io::Cursor;

use crate::constants::BINARY_COMPRESSION_LEVEL;

pub struct CompressedBytes(pub Vec<u8>);

impl CompressedBytes {
    pub fn as_u8_vec(self) -> Vec<u8> {
        self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }
}

impl TryFrom<Vec<u8>> for CompressedBytes {
    type Error = std::io::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let bytes = zstd::stream::encode_all(Cursor::new(value), BINARY_COMPRESSION_LEVEL)?;
        Ok(Self(bytes))
    }
}

impl TryFrom<CompressedBytes> for Vec<u8> {
    type Error = std::io::Error;

    fn try_from(value: CompressedBytes) -> Result<Self, Self::Error> {
        let bytes = zstd::stream::decode_all(value.0.as_slice())?;
        Ok(bytes)
    }
}
