use std::{io::Write, marker::PhantomData};

use flate2::{write::ZlibEncoder, Compression};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use zune_inflate::DeflateDecoder;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Compressed<T> {
    #[serde(with = "serde_bytes")]
    inner: Vec<u8>,
    _phantom: PhantomData<T>,
}

impl<T> Compressed<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn with_compression_level(compression: Compression, data: &T) -> postcard::Result<Self> {
        let uncompressed = postcard::to_allocvec(data)?;
        let mut encoder = ZlibEncoder::new(Vec::<u8>::new(), compression);
        encoder.write_all(&uncompressed).unwrap();
        let compressed = encoder.finish().unwrap();

        Ok(Self {
            inner: compressed,
            _phantom: PhantomData,
        })
    }

    pub fn new(data: &T) -> postcard::Result<Self> {
        Self::with_compression_level(Compression::default(), data)
    }

    pub fn decompress(&self) -> postcard::Result<T> {
        if let Ok(uncompressed) = DeflateDecoder::new(&self.inner).decode_zlib() {
            postcard::from_bytes(&uncompressed)
        } else {
            Err(postcard::Error::DeserializeBadEncoding)
        }
    }
}
