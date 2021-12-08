//! Common behavior on backing stores for Pigment caches.

use super::Error;

use std::ops::Deref;

type Result<T> = std::result::Result<T, Error>;

pub trait Store {
    type ByteVec: AsRef<[u8]> + Deref<Target = [u8]> + From<Vec<u8>> = Vec<u8>;

    /// Get a value from the store by its key.
    fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<Self::ByteVec>>;

    /// Insert a key-value pair into the store.
    fn insert<K: AsRef<[u8]>, V: Into<Self::ByteVec>>(
        &self,
        key: K,
        value: V,
    ) -> Result<Option<Self::ByteVec>>;

    type ScanPrefixIter: Iterator<Item = Result<(Self::ByteVec, Self::ByteVec)>> =
        Box<dyn Iterator<Item = Result<(Self::ByteVec, Self::ByteVec)>>>;

    /// Scan a prefix of the store.
    fn scan_prefix<P: AsRef<[u8]>>(&self, prefix: P) -> Self::ScanPrefixIter;

    /// Destroy all keys with the given prefix.
    fn destroy_prefix<P: AsRef<[u8]>>(&self, prefix: P) -> Result<()>;
}

impl Store for sled::Tree {
    type ByteVec = sled::IVec;

    fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<Self::ByteVec>> {
        self.get(key).map_err(Error::Sled)
    }

    fn insert<K: AsRef<[u8]>, V: Into<Self::ByteVec>>(
        &self,
        key: K,
        value: V,
    ) -> Result<Option<Self::ByteVec>> {
        self.insert(key, value).map_err(Error::Sled)
    }

    fn scan_prefix<P: AsRef<[u8]>>(&self, prefix: P) -> Self::ScanPrefixIter {
        Box::new(self.scan_prefix(prefix).map(|r| r.map_err(Error::Sled)))
    }

    fn destroy_prefix<P: AsRef<[u8]>>(&self, prefix: P) -> Result<()> {
        self.scan_prefix(prefix)
            .try_for_each(|kv| self.remove(kv?.0).map_err(Error::Sled).map(|_| ()))
    }
}
