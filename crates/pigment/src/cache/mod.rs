//! Caching and Querying Schemes for Canvas Resources

pub mod error;
pub mod key;
mod resource;
pub mod store;

use crate::{Result, View};

pub use error::Error;
pub use key::Key;
pub use store::Store;

use canvas::{DateTime, Resource};
use futures::prelude::*;
use std::{ops::RangeBounds, time::SystemTime};

pub trait Cache: Resource {
    type Key: Key;
    fn key(&self) -> Self::Key;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry<R> {
    pub resource: R,
    pub updated: DateTime,
}

/// Replace all resources in the cache under a given view with the given resources.
#[inline]
pub async fn replace_view<S: Store, R: Cache, E, RStream: Stream<Item = Result<R, E>> + Unpin>(
    store: &S,
    view: &View,
    resources: &mut RStream,
) -> Result<Result<(), E>> {
    tracing::trace!("replacing view {}", view);
    // the start of the gap between the preceding resource and the current one
    let mut gap_start: Vec<u8> = Vec::with_capacity(R::Key::SER_LEN);
    while let Some(res) = resources.next().await {
        match res {
            Ok(resource) => {
                let key = [view.serialize()?, resource.key().serialize()?].concat();

                if key > gap_start {
                    // remove all keys inbetween the last key and this key
                    // we use multiple [`Store::remove_range`] calls so that key writes are in-order,
                    // thereby improving performance for LSMT-based stores
                    store
                        .remove_range(gap_start.as_slice()..key.as_slice())
                        .await?;
                }

                // FIXME: only replace if more updated than existing
                store
                    .insert(
                        &key,
                        bincode::serialize(&CacheEntry {
                            resource,
                            updated: SystemTime::now().into(),
                        })
                        .map_err(Error::Serialization)?,
                    )
                    .await?;

                fn increment_key(key: &mut [u8]) {
                    if let Some((last, rest)) = key.split_last_mut() {
                        let (new, overflowed) = last.overflowing_add(1);
                        *last = new;
                        if overflowed {
                            increment_key(rest)
                        }
                    }
                }

                // move the key forward by one to get the start of the gap
                // this assumes that the keys will not increase in length
                gap_start = key;
                increment_key(&mut gap_start)
            }
            Err(e) => return Ok(Err(e)),
        }
    }

    Ok(Ok(()))
}

/// Get a single resource from the cache.
#[inline]
pub async fn get<S: Store, R: Cache>(
    store: &S,
    view: &View,
    key: &R::Key,
) -> Result<Option<CacheEntry<R>>> {
    let val = store
        .get(&[view.serialize()?, key.serialize()?].concat())
        .await?;

    val.map(|res| {
        bincode::deserialize(&res)
            .map_err(Error::Deserialization)
            .map_err(Into::into)
    })
    .transpose()
}

/// Get all resources under the view from the cache.
#[inline]
pub fn get_all<S: Store, R: Cache>(
    store: &S,
    view: &View,
) -> Result<impl Stream<Item = Result<(R::Key, CacheEntry<R>)>>> {
    Ok(store.scan_prefix(&view.serialize()?).map(|res| {
        let (key, val) = res?;

        let key = R::Key::deserialize(&mut key.iter().skip(View::SER_LEN).copied())?;

        let entry: CacheEntry<R> =
            bincode::deserialize(val.as_ref()).map_err(Error::Deserialization)?;

        Ok((key, entry))
    }))
}

#[inline]
pub fn prefix_to_range<P: AsRef<[u8]>>(prefix: P) -> Option<impl RangeBounds<P>>
where
    std::ops::Range<Vec<u8>>: RangeBounds<P>,
{
    let mut end = prefix.as_ref().to_vec();
    *end.last_mut()? += 1;

    Some(prefix.as_ref().to_vec()..end)
}