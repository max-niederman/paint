//! Common behavior on backing stores for Ebuache caches.

use super::Error;
use std::ops::RangeBounds;

type Result<T> = std::result::Result<T, Error>;

pub trait Store {
    /// The type of owned byte vectors inserted into the store.
    type ByteVec: AsRef<[u8]> + From<Vec<u8>> + Clone;
    /// The type of byte vectors borrowed from the store.
    type ByteVecBorrowed<'s>: AsRef<[u8]> + 's
    where
        Self: 's;

    /// Get a value from the store by its key.
    fn get<K: AsRef<[u8]>>(&self, key: &K) -> Result<Option<Self::ByteVecBorrowed<'_>>>;

    /// Insert a key-value pair into the store.
    fn insert<K: AsRef<[u8]>, V: Into<Self::ByteVec>>(
        &self,
        key: &K,
        value: V,
    ) -> Result<Option<Self::ByteVec>>;

    /// Remove a key-value pair from the store.
    fn remove<K: AsRef<[u8]>>(&self, key: &K) -> Result<Option<Self::ByteVec>>;

    type ScanRangeIter<'s, K, R>: Iterator<
        Item = Result<(Self::ByteVecBorrowed<'s>, Self::ByteVecBorrowed<'s>)>,
    >
    where
        Self: 's,
        Self::ByteVec: 's,
        K: 's,
        R: 's;
    /// Scan a range of keys in the store.
    fn scan_range<'s, K, R>(&'s self, range: R) -> Self::ScanRangeIter<'s, K, R>
    where
        K: AsRef<[u8]> + 's,
        R: RangeBounds<K> + 's;

    /// Remove a range of keys in the store.
    fn remove_range<K: AsRef<[u8]>, R: RangeBounds<K>>(&self, range: R) -> Result<()> {
        self.scan_range(range)
            .try_for_each(|kv| self.remove(&kv?.0).map(|_| ()))
    }

    type ScanPrefixIter<'s>: Iterator<
        Item = Result<(Self::ByteVecBorrowed<'s>, Self::ByteVecBorrowed<'s>)>,
    >
    where
        Self: 's,
        Self::ByteVec: 's;
    /// Scan a prefix of the store.
    fn scan_prefix<'s, P: AsRef<[u8]>>(&self, prefix: &P) -> Self::ScanPrefixIter<'_>;

    /// Remove all keys with the given prefix.
    fn remove_prefix<P: AsRef<[u8]>>(&self, prefix: &P) -> Result<()> {
        self.scan_prefix(prefix)
            .try_for_each(|kv| self.remove(&kv?.0).map(|_| ()))
    }
}
