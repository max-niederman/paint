//! Caching and Querying Schemes for Canvas Resources

pub mod error;
pub mod key;
pub mod store;

pub use error::Error;
pub use store::Store;

use crate::{Result, View};
use key::Key;

use canvas::{DateTime, Resource};
use futures::Stream;

pub trait Cache<S: Store>: Resource {
    type Key: Key;

    /// Atomically replace all resources in the cache under a given view with the given resources.
    fn replace_view<R: Stream<Item = Self>>(
        store: &S,
        view: &View,
        key: &Self::Key,
        resources: R,
    ) -> Result<()>;

    /// Get a single resource from the cache.
    fn get(store: &S, view: &View, key: &Self::Key) -> Result<Option<CacheEntry<Self>>>;
    /// Get all resources under the view from the cache.
    fn get_all<'s, 'v>(
        store: &'s S,
        view: &'v View,
    ) -> Result<Box<dyn 'v + Iterator<Item = Result<(Self::Key, CacheEntry<Self>)>>>>
    where
        S::ScanPrefixIter: 'v;
}

default impl<R: Resource, S: Store> Cache<S> for R {
    /// Atomically replace all resources in the cache under a given view with the given resources.
    fn replace_view<RStream: Stream<Item = Self>>(
        store: &S,
        view: &View,
        key: &Self::Key,
        resources: RStream,
    ) -> Result<()> {
        unimplemented!()
    }

    /// Get a single resource from the cache.
    fn get(store: &S, view: &View, key: &Self::Key) -> Result<Option<CacheEntry<Self>>> {
        let val = store.get([view.serialize()?, key.serialize()?].concat())?;

        val.map(|res| {
            bincode::deserialize(&res)
                .map_err(Error::Deserialization)
                .map_err(Into::into)
        })
        .transpose()
    }

    /// Get all resources under the view from the cache.
    fn get_all<'s, 'v>(
        store: &'s S,
        view: &'v View,
    ) -> Result<Box<dyn 'v + Iterator<Item = Result<(Self::Key, CacheEntry<Self>)>>>>
    where
        S::ScanPrefixIter: 'v,
    {
        Ok(Box::new(store.scan_prefix(view.serialize()?).map(|res| {
            let (key, val) = res?;

            let key = Self::Key::deserialize(&mut key.iter().copied())?;

            let entry: CacheEntry<Self> =
                bincode::deserialize(val.as_ref()).map_err(Error::Deserialization)?;

            Ok((key, entry))
        })))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry<R> {
    resource: R,
    updated: DateTime,
    last_accessed: Option<DateTime>,
}
