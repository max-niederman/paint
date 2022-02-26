use poem::{Error, Result};
use reqwest::StatusCode;
use uuid::Uuid;

/// A key uniquely identifying a resource in a cache.
pub trait Key: Sized + Send + Sync {
    /// The length in bytes of the serialized key.
    const SER_LEN: usize;

    /// Serialize the key into a byte vector.
    ///
    /// Unfortunately, because Rust does not have full-blown dependent typing (yet),
    /// we cannot just return a [`[u8; Self::SER_LEN]`] here because we would then not be
    /// able to implement [`Key`] for some composite keys. Nevertheless, the returned
    /// [`Vec`] must always be have length [`SER_LEN`].
    fn key_serialize(&self) -> Result<Vec<u8>>;

    /// Deserialize the key from a byte iterator.
    fn key_deserialize<I: IntoIterator<Item = u8>>(bytes: I) -> Result<Self>;
}

impl Key for Uuid {
    const SER_LEN: usize = 16;

    fn key_serialize(&self) -> Result<Vec<u8>> {
        Ok(self.as_bytes().to_vec())
    }

    fn key_deserialize<I: IntoIterator<Item = u8>>(bytes: I) -> Result<Self, poem::Error> {
        let bytes = bytes.into_iter();
        Ok(Self::from_bytes(
            bytes
                .take(Self::SER_LEN)
                .collect::<heapless::Vec<_, { Self::SER_LEN }>>()
                .into_array()
                .map_err(|_| {
                    Error::from_string(
                        "uuid key byte iterator was too short",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )
                })?,
        ))
    }
}

impl Key for canvas::Id {
    const SER_LEN: usize = std::mem::size_of::<canvas::Id>();

    fn key_serialize(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn key_deserialize<I: IntoIterator<Item = u8>>(bytes: I) -> Result<Self> {
        let bytes = bytes.into_iter();
        Ok(Self::from_be_bytes(
            bytes
                .take(Self::SER_LEN)
                .collect::<heapless::Vec<_, { Self::SER_LEN }>>()
                .into_array()
                .map_err(|_| {
                    Error::from_string(
                        "canvas id key byte iterator was too short",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )
                })?,
        ))
    }
}
