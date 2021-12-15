//! Common behavior on backing stores for Pigment caches.

#[cfg(feature = "sled")]
pub mod sled;

use super::Error;
use futures::{Future, Stream};
use std::ops::{Deref, RangeBounds};

type Result<T> = std::result::Result<T, Error>;

pub trait Store {
    /// The type of byte vectors retrieved from and inserted into the store.
    type ByteVec: AsRef<[u8]> + Deref<Target = [u8]> + From<Vec<u8>>;

    type GetFut: Future<Output = Result<Option<Self::ByteVec>>>;

    /// Get a value from the store by its key.
    fn get<K: AsRef<[u8]>>(&self, key: &K) -> Self::GetFut;

    type InsertFut: Future<Output = Result<Option<Self::ByteVec>>>;

    /// Insert a key-value pair into the store.
    fn insert<K: AsRef<[u8]>, V: Into<Self::ByteVec>>(
        &self,
        key: &K,
        value: Self::ByteVec,
    ) -> Self::InsertFut;

    type RemoveFut: Future<Output = Result<Option<Self::ByteVec>>>;

    /// Remove a key-value pair from the store.
    fn remove<K: AsRef<[u8]>>(&self, key: &K) -> Self::RemoveFut;

    type ScanRangeStream: Stream<Item = Result<(Self::ByteVec, Self::ByteVec)>>;

    /// Scan a range of keys in the store.
    fn scan_range<K: AsRef<[u8]>, R: RangeBounds<K>>(&self, range: R) -> Self::ScanRangeStream;

    type RemoveRangeFut: Future<Output = Result<()>>;

    /// Remove a range of keys in the store.
    fn remove_range<K: AsRef<[u8]>, R: RangeBounds<K>>(&self, range: R) -> Self::RemoveRangeFut;

    type ScanPrefixStream: Stream<Item = Result<(Self::ByteVec, Self::ByteVec)>>;

    /// Scan a prefix of the store.
    fn scan_prefix<P: AsRef<[u8]>>(&self, prefix: &P) -> Self::ScanPrefixStream;

    type RemovePrefixFut: Future<Output = Result<()>>;

    /// Remove all keys with the given prefix.
    fn remove_prefix<P: AsRef<[u8]>>(&self, prefix: &P) -> Self::RemovePrefixFut;
}
