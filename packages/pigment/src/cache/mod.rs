//! Caching and Querying Schemes for Canvas Resources

pub mod error;
pub mod key;
mod resource;
pub mod store;

pub use error::{Error, Result};
pub use key::Key;
pub use store::Store;

use crate::View;
use canvas::{DateTime, Resource};
use futures::prelude::*;
use std::{ops::RangeBounds, time::SystemTime};

pub trait Cache: Resource + PartialEq {
    type Key: Key;
    fn key(&self) -> Self::Key;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry<R> {
    pub resource: R,
    pub updated: DateTime,
    pub written: DateTime,
}

/// Replace all resources in the cache under a given view with the given resources.
/// The resource stream must yield resources in key-lexicographical order, so that we can efficiently write to the store.
#[inline]
#[tracing::instrument(skip(store, resources), fields(resource = std::any::type_name::<R>()))]
pub async fn replace_view_ordered<S, R, RStream, E>(
    store: &S,
    view: &View,
    resources: &mut RStream,
) -> Result<Result<(), E>>
where
    S: Store,
    R: Cache,
    RStream: Stream<Item = Result<(R::Key, Option<R>), E>> + Unpin,
{
    // the start of the gap between the preceding resource and the current one
    let mut gap_start = view.serialize()?;

    while let Some(res) = resources.next().await {
        match res {
            Ok((key, resource)) => {
                let key_bytes = [view.serialize()?, key.serialize()?].concat();

                if key_bytes >= gap_start {
                    // remove all keys inbetween the last key and this key
                    store
                        .remove_range(gap_start.as_slice()..key_bytes.as_slice())
                        .await?;
                } else {
                    return Err(Error::UnexpectedStreamYield {
                        expected: "key lexicographically greater than the last",
                        actual: "key lexicographically less than the last",
                    });
                }

                let old = store
                    .get(&key_bytes)
                    .await?
                    .map(|bytes| {
                        bincode::deserialize::<CacheEntry<R>>(bytes.as_ref())
                            .map_err(Error::Deserialization)
                    })
                    .transpose()?;

                if let Some(resource) = resource {
                    let now: DateTime = SystemTime::now().into();
                    store
                        .insert(
                            &key_bytes,
                            bincode::serialize(&CacheEntry {
                                updated: now,
                                written: match old {
                                    Some(CacheEntry {
                                        written,
                                        resource: old_resource,
                                        ..
                                    }) if old_resource == resource => written,
                                    _ => SystemTime::now().into(),
                                },
                                resource,
                            })
                            .map_err(Error::Serialization)?,
                        )
                        .await?;
                } else if let Some(old) = old {
                    store
                        .insert(
                            &key_bytes,
                            bincode::serialize(&CacheEntry {
                                updated: SystemTime::now().into(),
                                ..old
                            })
                            .map_err(Error::Serialization)?,
                        )
                        .await?;
                }

                // move the key forward by one to get the start of the gap
                // this assumes that the keys will not increase in length
                gap_start = key_bytes;
                increment_key(&mut gap_start);
            }
            Err(e) => return Ok(Err(e)),
        }
    }

    let mut end = view.serialize()?;
    end.extend(std::iter::repeat(0xFF).take(R::Key::SER_LEN));
    store
        .remove_range(gap_start.as_slice()..end.as_slice())
        .await?;

    Ok(Ok(()))
}

/// Get a single resource from the cache.
#[inline]
#[tracing::instrument(skip(store, key), fields(resource = std::any::type_name::<R>()))]
pub async fn get<S: Store, R: Cache>(
    store: &S,
    view: &View,
    key: &R::Key,
) -> Result<Option<CacheEntry<R>>> {
    let val = store
        .get(&[view.serialize()?, key.serialize()?].concat())
        .await?;

    val.map(|bytes| bincode::deserialize::<CacheEntry<R>>(&bytes.as_ref()).map_err(Error::Deserialization))
        .transpose()
}

/// Get all resources under the view from the cache.
#[inline]
#[tracing::instrument(skip(store), fields(resource = std::any::type_name::<R>()))]
pub fn get_all<S: Store, R: Cache>(
    store: &S,
    view: &View,
) -> Result<impl Stream<Item = Result<(R::Key, CacheEntry<R>)>>> {
    Ok(store.scan_prefix(&view.serialize()?).map(|res| {
        let (key, val) = res?;

        let key = R::Key::deserialize(&mut key.as_ref().iter().skip(View::SER_LEN).copied())?;

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
