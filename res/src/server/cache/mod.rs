//! Caching and Querying Schemes for Canvas Resources

pub mod key;

use key::*;

use canvas::{DateTime, Resource};
use miette::{IntoDiagnostic, Result, WrapErr};
use pigment::View;
use serde::{Deserialize, Serialize};
use sled::Tree;

pub trait Cache: Resource {
    /// The key type represents how resources are stored under each [`View`].
    type Key: Key;

    /// Atomically replace all resources in the cache under a given view with the given resources.
    fn replace_view(tree: &Tree, view: &View, key: &Self::Key, resource: Self) -> Result<()> {
        unimplemented!()
    }

    /// Get a single resource from the cache.
    fn get(tree: &Tree, view: &View, key: &Self::Key) -> Result<Option<CacheEntry<Self>>> {
        let val = tree
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
    fn get_all<'t, 'v, P: KeyPrefix<Self::Key>>(
        tree: &'t Tree,
        view: &'v View,
        prefix: &P,
    ) -> Box<dyn 'v + Iterator<Item = Result<(Self::Key, CacheEntry<Self>)>>> {
        Box::new(tree.scan_prefix(Join(view, prefix).as_bytes()).map(|res| {
            let (key, val) = res.into_diagnostic()?;

            let key = Self::Key::from_bytes(&key)?;

            let entry: CacheEntry<Self> = bincode::deserialize(val.as_ref())
                .into_diagnostic()
                .wrap_err("while deserializing entry")?;

            Ok((key, entry))
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<R> {
    resource: R,
    updated: DateTime,
    last_accessed: Option<DateTime>,
}
