use crossbeam_skiplist::{map, SkipMap};
use pigment::cache::*;
use std::ops::{Bound, Range, RangeBounds};

#[derive(Debug)]
pub struct GlazeStore {
    resources: SkipMap<Vec<u8>, Vec<u8>>,
}

impl Store for GlazeStore {
    type ByteVec = Vec<u8>;
    type ByteVecBorrowed<'s> = Vec<u8>;

    fn get<K: AsRef<[u8]>>(&self, key: &K) -> Result<Option<Self::ByteVecBorrowed<'_>>> {
        Ok(self
            .resources
            .get(key.as_ref())
            .as_ref()
            .map(map::Entry::value)
            .cloned())
    }

    /// [`SkipMap`] does not return the previous value on insertion, so we always return `Ok(None)`.
    fn insert<K: AsRef<[u8]>, V: Into<Self::ByteVec>>(
        &self,
        key: &K,
        value: V,
    ) -> Result<Option<Self::ByteVec>> {
        self.resources.insert(key.as_ref().to_vec(), value.into());
        Ok(None)
    }

    /// [`SkipMap`] does not return the previous value on removal, so we always return `Ok(None)`.
    fn remove<K: AsRef<[u8]>>(&self, key: &K) -> Result<Option<Self::ByteVec>> {
        self.resources.remove(key.as_ref());
        Ok(None)
    }

    type ScanRangeIter<'s, K: 's, R: 's> = GlazeIter<'s, (Bound<Vec<u8>>, Bound<Vec<u8>>)>;
    fn scan_range<'s, K, R>(&'s self, range: R) -> Self::ScanRangeIter<'s, K, R>
    where
        K: AsRef<[u8]> + 's,
        R: RangeBounds<K> + 's,
    {
        // I tried to avoid a clone here for literally an hour, and it's just not worth it
        GlazeIter(self.resources.range((
            range.start_bound().map(|bound| bound.as_ref().to_vec()),
            range.end_bound().map(|bound| bound.as_ref().to_vec()),
        )))
    }

    type ScanPrefixIter<'s> = Self::ScanRangeIter<'s, Vec<u8>, Range<Vec<u8>>>;
    fn scan_prefix<P: AsRef<[u8]>>(&self, prefix: &P) -> Self::ScanPrefixIter<'_> {
        let start: Vec<u8> = prefix.as_ref().to_vec();
        let end = {
            let mut end = prefix.as_ref().to_vec();
            increment_key(&mut end);
            end
        };

        self.scan_range(start..end)
    }
}

pub struct GlazeIter<'s, R: RangeBounds<Vec<u8>>>(map::Range<'s, Vec<u8>, R, Vec<u8>, Vec<u8>>);
impl<'s, R> Iterator for GlazeIter<'s, R>
where
    R: RangeBounds<Vec<u8>>,
{
    type Item = Result<(Vec<u8>, Vec<u8>)>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|entry| Ok((entry.key().clone(), entry.value().clone())))
    }
}
