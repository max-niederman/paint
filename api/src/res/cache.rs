use super::Fetch;
use crate::{res::Resource, view::View};
use chrono::{DateTime, Duration, Utc};
use futures::prelude::*;
use miette::Diagnostic;
use poem::error::ResponseError;
use serde::{Deserialize, Serialize};
use sled::IVec;

pub struct Cache {
    db: sled::Db,
    max_age: Duration,
}

impl Cache {
    pub fn new(path: &impl AsRef<std::path::Path>, max_age: u64) -> Result<Self> {
        Ok(Self {
            db: sled::open(path)?,
            max_age: Duration::seconds(max_age as _),
        })
    }
}

impl Cache {
    #[tracing::instrument(skip(self, client, view, collection))]
    pub async fn cached_fetch<'f, F, C>(
        &self,
        client: C,
        view: &'f View,
        collection: &'f F,
    ) -> Result<sled::Iter>
    where
        F: Fetch<'f, C>,
        Error: From<<F as Fetch<'f, C>>::Err>,
    {
        // TODO: should this be an atomic transaction since we're working on two trees?

        let prefix = collection.cache_prefix(view);
        let entry_tree = self.db.open_tree(prefix.space)?;
        let meta_tree = self.db.open_tree(format!("{}_meta", prefix.space))?;

        let meta = meta_tree
            .get(&prefix.key)?
            .map(|bytes| bincode::deserialize::<CacheMeta>(&bytes))
            .transpose()
            .map_err(Error::EntryMetaDeserialization)?;

        if let Some(meta) = meta {
            if Utc::now() - meta.inserted < self.max_age {
                tracing::debug!("cache hit and fresh");

                let mut upper_bound = prefix.key.clone();
                increment_key(&mut upper_bound);

                return Ok(entry_tree.range(prefix.key..upper_bound));
            }

            tracing::debug!("cache hit but stale");
        } else {
            tracing::debug!("cache miss");
        }

        tracing::debug!("fetching collection from Canvas");

        let meta_serialized = bincode::serialize(&CacheMeta {
            inserted: Utc::now(),
        })
        .map_err(Error::EntryMetaSerialization)?;

        // FIXME: this should almost certainly be a multi-tree transaction

        let mut resource_stream = collection.fetch_all(view, client);
        while let Some(resource) = resource_stream.next().await {
            let resource = resource?;
            let location = resource.cache_location(view);
            assert_eq!(location.space, prefix.space);

            entry_tree.insert(
                &location.key,
                serde_json::to_vec(&resource).map_err(Error::EntrySerialization)?,
            )?;
            meta_tree.insert(&location.key, meta_serialized.as_slice())?;
        }
        meta_tree.insert(&prefix.key, meta_serialized)?;

        let mut upper_bound = prefix.key.clone();
        increment_key(&mut upper_bound);
        Ok(entry_tree.range(prefix.key..upper_bound))
    }

    #[tracing::instrument(skip(self, client, view, collection))]
    pub async fn cached_fetch_one<'f, F, C>(
        &self,
        client: C,
        view: &'f View,
        collection: &'f F,
    ) -> Result<Option<IVec>>
    where
        F: Fetch<'f, C>,
        Error: From<<F as Fetch<'f, C>>::Err>,
    {
        // TODO: should this be an atomic transaction since we're working on two trees?

        let location = collection.cache_prefix(view);
        let entry_tree = self.db.open_tree(location.space)?;
        let meta_tree = self.db.open_tree(format!("{}_meta", location.space))?;

        let meta = meta_tree
            .get(&location.key)?
            .map(|bytes| bincode::deserialize::<CacheMeta>(&bytes))
            .transpose()
            .map_err(Error::EntryMetaDeserialization)?;

        if let Some(meta) = meta {
            if Utc::now() - meta.inserted > self.max_age {
                let mut upper_bound = location.key.clone();
                increment_key(&mut upper_bound);
                return Ok(entry_tree
                    .range(location.key..upper_bound)
                    .next()
                    .transpose()?
                    .map(|(_k, v)| v));
            }
        }

        // FIXME: this should almost certainly be a multi-tree transaction

        let resource = collection
            .fetch_all(view, client)
            .next()
            .await
            .transpose()?;
        entry_tree.insert(
            &location.key,
            serde_json::to_vec(&resource).map_err(Error::EntrySerialization)?,
        )?;
        meta_tree.insert(
            &location.key,
            bincode::serialize(&CacheMeta {
                inserted: Utc::now(),
            })
            .map_err(Error::EntryMetaSerialization)?,
        )?;

        let mut upper_bound = location.key.clone();
        increment_key(&mut upper_bound);
        Ok(entry_tree
            .range(location.key..upper_bound)
            .next()
            .transpose()?
            .map(|(_k, v)| v))
    }
}

/// Metadata about a cache entry.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CacheMeta {
    inserted: DateTime<Utc>,
}

#[inline]
pub fn increment_key(key: &mut [u8]) {
    if let Some((last, rest)) = key.split_last_mut() {
        let (new, overflowed) = last.overflowing_add(1);
        *last = new;
        if overflowed {
            increment_key(rest)
        }
    }
}

#[test]
fn increments_key() {
    macro_rules! test {
        ($($key:expr => $expected:expr),*,) => {
            $({
                let mut key = $key.to_vec();
                increment_key(&mut key);
                assert_eq!(&key, &$expected);
            })*
        }
    }

    test!(
        [0u8; 0] => [0u8; 0],
        [0x0] => [0x1],
        [0x0, 0x0] => [0x0, 0x1],
        [0x0, 0xFF] => [0x1, 0x0],
        [0x0, 0xFF, 0xFF] => [0x1, 0x0, 0x0],
    );
}

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    Sled(#[from] sled::Error),

    #[error(transparent)]
    Canvas(#[from] canvas_lms::client::Error),

    #[error("entry meta could not be serialized")]
    EntryMetaSerialization(#[source] bincode::Error),

    #[error("entry meta could not be deserialized")]
    EntryMetaDeserialization(#[source] bincode::Error),

    #[error("entry could not be serialized to JSON")]
    EntrySerialization(#[source] serde_json::Error),
}

impl ResponseError for Error {
    fn status(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
