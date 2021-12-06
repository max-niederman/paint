//! Caching and Querying Schemes for Canvas Resources

pub mod error;
pub mod key;

pub use error::Error;

use crate::View;
use key::Key;

use canvas::{DateTime, Resource};
use futures::Stream;
use miette::{IntoDiagnostic, Result, WrapErr};
use sled::Tree;

pub trait Cache<S: Store>: Resource {
    type Key: Key;

    // FIXME: return concrete error types

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
    ) -> Result<Box<dyn 'v + Iterator<Item = Result<(Self::Key, CacheEntry<Self>)>>>>;
}

default impl<R: Resource> Cache<Tree> for R {
    /// Atomically replace all resources in the cache under a given view with the given resources.
    fn replace_view<RStream: Stream<Item = Self>>(
        store: &Tree,
        view: &View,
        key: &Self::Key,
        resources: RStream,
    ) -> Result<()> {
        unimplemented!()
    }

    /// Get a single resource from the cache.
    fn get(store: &Tree, view: &View, key: &Self::Key) -> Result<Option<CacheEntry<Self>>> {
        let val = store
            .get([view.serialize(), key.serialize()].concat())
            .into_diagnostic()
            .wrap_err("while getting entry")?;

        val.map(|res| {
            bincode::deserialize(&res)
                .into_diagnostic()
                .wrap_err("while deserializing entry")
        })
        .transpose()
    }

    /// Get all resources matching the key from the cache.
    fn get_all<'s, 'v>(
        store: &'s Tree,
        view: &'v View,
    ) -> Result<Box<dyn 'v + Iterator<Item = Result<(Self::Key, CacheEntry<Self>)>>>> {
        Ok(Box::new(store.scan_prefix(view.serialize()?).map(|res| {
            let (key, val) = res.into_diagnostic()?;

            let key = Self::Key::deserialize(&mut key.iter().copied())?;

            let entry: CacheEntry<Self> = bincode::deserialize(val.as_ref())
                .into_diagnostic()
                .wrap_err("while deserializing entry")?;

            Ok((key, entry))
        })))
    }
}

trait Store {}
impl Store for Tree {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry<R> {
    resource: R,
    updated: DateTime,
    last_accessed: Option<DateTime>,
}
