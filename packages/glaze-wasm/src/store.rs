use crossbeam_skiplist::{map, SkipMap};
use indexed_db_futures::prelude::*;
use pigment::cache::*;
use std::ops::{Bound, RangeBounds};

use web_sys::DomException;

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

    type ScanRangeIter<'s> = GlazeIter<'s, (Bound<Vec<u8>>, Bound<Vec<u8>>)>;
    fn scan_range<'s, K, R>(&'s self, range: R) -> Self::ScanRangeIter<'s>
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

    type ScanPrefixIter<'s> = Self::ScanRangeIter<'s>;
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

const IDB_NAME: &str = "glaze_store";
const IDB_VERSION: u32 = 1;

impl GlazeStore {
    /// Load the [`GlazeStore`] from IndexedDB.
    pub async fn load(_name: &str) -> Result<Self, DomException> {
        todo!()
    }

    /// Write the [`GlazeStore`] to IndexedDB.
    pub async fn write(&self) -> Result<(), DomException> {
        todo!()
    }
}

async fn get_database() -> Result<IdbDatabase, DomException> {
    let mut req: OpenDbRequest = IdbDatabase::open_u32(IDB_NAME, IDB_VERSION)?;
    // if the store is outdated, we delete all of its data
    req.set_on_upgrade_needed(Some(|event: &IdbVersionChangeEvent| {
        for name in event.db().object_store_names() {
            event.db().delete_object_store(&name)?;
        }
        Ok(())
    }));
    req.into_future().await
}
