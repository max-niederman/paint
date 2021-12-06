//! Common behavior on backing stores for Pigment caches.

use super::Error;

use std::ops::Deref;

type Result<T> = std::result::Result<T, Error>;

pub trait Store {
    type ByteVec: AsRef<[u8]> + Deref<Target = [u8]> + Into<Vec<u8>> = Vec<u8>;

    fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<Self::ByteVec>>;

    type ScanPrefixIter: Iterator<Item = Result<(Self::ByteVec, Self::ByteVec)>>;
    fn scan_prefix<P: AsRef<[u8]>>(&self, prefix: P) -> Self::ScanPrefixIter;
}

impl Store for sled::Tree {
    type ByteVec = Vec<u8>;

    fn get<K: AsRef<[u8]>>(&self) -> Result<Option<Self::ByteVec>> {
        todo!()
    }

    type ScanPrefixIter;

    fn scan_prefix<P: AsRef<[u8]>>(&self, prefix: P) -> Self::ScanPrefixIter {
        todo!()
    }
}
