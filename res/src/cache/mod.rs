//! Caching and Querying Schemes for Canvas Resources

pub mod error;
pub mod key;
pub mod store;

use std::time::SystemTime;

pub use error::Error;
pub use store::Store;

use crate::{Result, View};
use key::Key;

use canvas::{DateTime, Resource};
use futures::{Stream, StreamExt};

pub trait Cache: Resource {
    type Key: Key;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry<R> {
    resource: R,
    updated: DateTime,
    last_accessed: Option<DateTime>,
}

/// Atomically replace all resources in the cache under a given view with the given resources.
async fn replace_view<S: Store, R: Cache, RStream: Stream<Item = R> + Unpin>(
    store: &S,
    view: &View,
    resources: &mut RStream,
) -> Result<()> {
    store.destroy_prefix(view.serialize()?)?;

    for resource in resources.next().await {
        store.insert(view.serialize()?, bincode::serialize(&CacheEntry {
            resource,
            updated: SystemTime::now().into(),
            last_accessed: None,
        }).map_err(Error::Serialization)?)?;
    }

    Ok(())
}

/// Get a single resource from the cache.
async fn get<S: Store, R: Cache>(
    store: &S,
    view: &View,
    key: &R::Key,
) -> Result<Option<CacheEntry<R>>> {
    let val = store.get([view.serialize()?, key.serialize()?].concat())?;

    val.map(|res| {
        bincode::deserialize(&res)
            .map_err(Error::Deserialization)
            .map_err(Into::into)
    })
    .transpose()
}

/// Get all resources under the view from the cache.
async fn get_all<'s, 'v, S: Store, R: Cache>(
    store: &'s S,
    view: &'v View,
) -> Result<impl 'v + Iterator<Item = Result<(R::Key, CacheEntry<R>)>>>
where
    S::ScanPrefixIter: 'v,
{
    Ok(store.scan_prefix(view.serialize()?).map(|res| {
        let (key, val) = res?;

        let key = R::Key::deserialize(&mut key.iter().skip(View::SER_LEN).copied())?;

        let entry: CacheEntry<R> =
            bincode::deserialize(val.as_ref()).map_err(Error::Deserialization)?;

        Ok((key, entry))
    }))
}
