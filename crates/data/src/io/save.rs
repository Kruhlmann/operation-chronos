use crate::{
    constants::{FILE_MAGIC_BYTES_MAP, GAME_VERSION_BINARY},
    io::CompressedBytes,
};

pub enum SaveableFormat {
    Map(CompressedBytes),
}

impl SaveableFormat {
    pub fn from_read_bytes(
        header: &[u8; 8],
        version: &[u8; 3],
        body: &[u8],
    ) -> std::io::Result<Self> {
        if *version != *GAME_VERSION_BINARY {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "version mismatch - found {version:#?} expected {:#?}",
                    *GAME_VERSION_BINARY
                ),
            ));
        }

        match header {
            b if *b == FILE_MAGIC_BYTES_MAP => Ok(Self::Map(CompressedBytes(body.to_vec()))),
            invalid_header => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid header data: {:#?}", invalid_header),
            )),
        }
    }

    pub fn decompress(self) -> Result<Vec<u8>, std::io::Error> {
        // let target = match self {
        //     SaveableFormat::Map(b) => b,
        // };
        let SaveableFormat::Map(target) = self;
        let bytes: Vec<u8> = target.try_into()?;
        Ok(bytes)
    }

    pub fn file_extension(&self) -> &str {
        match self {
            SaveableFormat::Map(..) => "ocmap",
        }
    }

    pub fn header_bytes(&self) -> [u8; 8] {
        match self {
            SaveableFormat::Map(..) => FILE_MAGIC_BYTES_MAP,
        }
    }

    pub fn body_bytes(self) -> Vec<u8> {
        match self {
            SaveableFormat::Map(b) => b.as_u8_vec(),
        }
    }
}

impl std::fmt::Display for SaveableFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveableFormat::Map(bytes) => {
                write!(f, "Map({} bytes)", bytes.len())
            }
        }
    }
}
