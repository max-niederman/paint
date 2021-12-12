//! Common behavior on backing stores for Pigment caches.

use super::Error;

use std::ops::{Deref, RangeBounds};

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

    /// Remove a key-value pair from the store.
    fn remove<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<Self::ByteVec>>;

    type ScanRangeIter: Iterator<Item = Result<(Self::ByteVec, Self::ByteVec)>> =
        Box<dyn Iterator<Item = Result<(Self::ByteVec, Self::ByteVec)>>>;

    /// Scan a range of keys in the store.
    fn scan_range<K: AsRef<[u8]>, R: RangeBounds<K>>(&self, range: R) -> Self::ScanRangeIter;

    /// Remove a range of keys in the store.
    fn remove_range<K: AsRef<[u8]>, R: RangeBounds<K>>(&self, range: R) -> Result<()> {
        self.scan_range(range)
            .try_for_each(|res| self.remove(res?.0).map(|_| ()))
    }

    type ScanPrefixIter: Iterator<Item = Result<(Self::ByteVec, Self::ByteVec)>> =
        Self::ScanRangeIter;

    /// Scan a prefix of the store.
    fn scan_prefix<P: AsRef<[u8]>>(&self, prefix: P) -> Self::ScanPrefixIter;

    /// Remove all keys with the given prefix.
    fn remove_prefix<P: AsRef<[u8]>>(&self, prefix: P) -> Result<()> {
        self.scan_prefix(prefix)
            .try_for_each(|res| self.remove(res?.0).map(|_| ()))
    }
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

    fn remove<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<Self::ByteVec>> {
        self.remove(key).map_err(Error::Sled)
    }

    type ScanRangeIter = SledIter;
    fn scan_range<K: AsRef<[u8]>, R: RangeBounds<K>>(&self, range: R) -> Self::ScanRangeIter {
        self.range(range).into()
    }

    fn scan_prefix<P: AsRef<[u8]>>(&self, prefix: P) -> Self::ScanPrefixIter {
        self.scan_prefix(prefix).into()
    }
}

pub struct SledIter(sled::Iter);

impl Iterator for SledIter {
    type Item = Result<(sled::IVec, sled::IVec)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|kv| kv.map_err(Error::Sled))
    }
}

impl From<sled::Iter> for SledIter {
    fn from(iter: sled::Iter) -> Self {
        SledIter(iter)
    }
}
