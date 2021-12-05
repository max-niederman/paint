//! Caching and Querying Schemes for Canvas Resources

pub mod key;

use key::*;

use canvas::{DateTime, Resource};
use miette::{IntoDiagnostic, Result, WrapErr};
use pigment::View;
use serde::{Deserialize, Serialize};
use sled::Tree;

pub trait Cache<S: Store>: Resource {
    /// The key type represents how resources are stored under each [`View`].
    type Key: Key;

    /// Atomically replace all resources in the cache under a given view with the given resources.
    fn replace_view(store: &S, view: &View, key: &Self::Key, resource: Self) -> Result<()>;

    /// Get a single resource from the cache.
    fn get(store: &S, view: &View, key: &Self::Key) -> Result<Option<CacheEntry<Self>>>;
    /// Get all resources matching the key from the cache.
    fn get_all<'s, 'v, P: KeyPrefix<Self::Key>>(
        store: &'s S,
        view: &'v View,
        prefix: &P,
    ) -> Box<dyn 'v + Iterator<Item = Result<(Self::Key, CacheEntry<Self>)>>>;
}

default impl<R: Resource> Cache<Tree> for R {
    /// Atomically replace all resources in the cache under a given view with the given resources.
    fn replace_view(store: &Tree, view: &View, key: &Self::Key, resource: Self) -> Result<()> {
        unimplemented!()
    }

    /// Get a single resource from the cache.
    fn get(store: &Tree, view: &View, key: &Self::Key) -> Result<Option<CacheEntry<Self>>> {
        let val = store
            .get(Join(view, key).as_bytes())
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
    fn get_all<'s, 'v, P: KeyPrefix<Self::Key>>(
        store: &'s Tree,
        view: &'v View,
        prefix: &P,
    ) -> Box<dyn 'v + Iterator<Item = Result<(Self::Key, CacheEntry<Self>)>>> {
        Box::new(store.scan_prefix(Join(view, prefix).as_bytes()).map(|res| {
            let (key, val) = res.into_diagnostic()?;

            let key = Self::Key::from_bytes(&key)?;

            let entry: CacheEntry<Self> = bincode::deserialize(val.as_ref())
                .into_diagnostic()
                .wrap_err("while deserializing entry")?;

            Ok((key, entry))
        }))
    }
}

trait Store {}
impl Store for Tree {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<R> {
    resource: R,
    updated: DateTime,
    last_accessed: Option<DateTime>,
}
