pub mod oil;
pub mod stores;

pub use stores::Stores;

use crossbeam_skiplist::{map, SkipMap};
use indexed_db_futures::prelude::*;
use js_sys::Uint8Array;
use pigment::cache::*;
use std::{
    fmt::{self, Debug},
    ops::{Bound, RangeBounds},
};

use web_sys::DomException;

pub struct GlazeStore {
    name: StoreName,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreName {
    Course,
    Assignment,
    Submission,
}

impl StoreName {
    const ALL: [Self; 3] = [Self::Course, Self::Assignment, Self::Submission];

    fn as_str(&self) -> &'static str {
        match self {
            Self::Course => "course",
            Self::Assignment => "assignment",
            Self::Submission => "submission",
        }
    }
}

impl GlazeStore {
    // TODO: by my best count, these methods have _three_ copies of the same data in memory;
    //       it would be nice to avoid that, considering each store could easily be megabytes.
    // TODO: load and write multiple stores to IndexedDB with the same transaction?

    /// Load the [`GlazeStore`] from IndexedDB.
    #[tracing::instrument]
    pub async fn load(name: StoreName) -> Result<Self, DomException> {
        let db: IdbDatabase = get_database().await?;
        let tr: IdbTransaction =
            db.transaction_on_one_with_mode(name.as_str(), IdbTransactionMode::Readonly)?;
        let bytes: Option<Uint8Array> = tr
            .object_store(name.as_str())?
            .get_owned("resources")?
            .await?
            .map(Into::into);

        if let Some(bytes) = bytes {
            Ok(Self {
                name,
                resources: Deserializer::new(bytes.to_vec().into_iter()).collect(),
            })
        } else {
            Ok(Self {
                name,
                resources: SkipMap::new(),
            })
        }
    }

    /// Write the [`GlazeStore`] to IndexedDB.
    #[tracing::instrument]
    pub async fn write(&self) -> Result<(), DomException> {
        let mut bytes = Vec::new();
        for entry in self.resources.iter() {
            bytes.extend_from_slice(&(entry.key().len() as u32).to_be_bytes());
            bytes.extend_from_slice(&(entry.value().len() as u32).to_be_bytes());
            bytes.extend_from_slice(entry.key());
            bytes.extend_from_slice(entry.value());
        }

        let db: IdbDatabase = get_database().await?;
        let tr: IdbTransaction =
            db.transaction_on_one_with_mode(self.name.as_str(), IdbTransactionMode::Readwrite)?;
        tr.object_store(self.name.as_str())?
            .put_key_val_owned("resources", &Uint8Array::from(bytes.as_slice()))?
            .into_future()
            .await?;

        Ok(())
    }
}

impl Debug for GlazeStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GlazeStore({})", self.name.as_str())
    }
}
#[tracing::instrument]
pub async fn get_database() -> Result<IdbDatabase, DomException> {
    tracing::trace!("creating IDB open request");
    let mut req: OpenDbRequest = IdbDatabase::open_u32(IDB_NAME, IDB_VERSION)?;
    // FIXME: if the store is outdated, we delete all of its data
    //        in the future, we also need to replace this data with a new copy from Oil
    req.set_on_upgrade_needed(Some(|event: &IdbVersionChangeEvent| {
        let _span = tracing::info_span!("upgrade store").entered();

        let db = event.db();

        tracing::debug!(
            "deleting old stores: {:?}",
            db.object_store_names().collect::<Vec<_>>()
        );
        for name in db.object_store_names() {
            db.delete_object_store(&name)?;
        }

        for name in StoreName::ALL {
            db.create_object_store(name.as_str())?;
        }
        Ok(())
    }));
    tracing::debug!("opening database");
    req.into_future().await
}
struct Deserializer<B> {
    bytes: B,
}

impl<B: Iterator<Item = u8>> Deserializer<B> {
    fn new(bytes: B) -> Self {
        Self { bytes }
    }

    fn consume_u32(&mut self) -> Option<u32> {
        self.bytes
            .by_ref()
            .take(4)
            .collect::<heapless::Vec<_, 4>>()
            .into_array()
            .ok()
            .map(u32::from_be_bytes)
    }
}

impl<B: Iterator<Item = u8>> Iterator for Deserializer<B> {
    type Item = (Vec<u8>, Vec<u8>);
    fn next(&mut self) -> Option<Self::Item> {
        let (key_len, val_len) = (self.consume_u32()?, self.consume_u32()?);
        Some((
            self.bytes.by_ref().take(key_len as usize).collect(),
            self.bytes.by_ref().take(val_len as usize).collect(),
        ))
    }
}
