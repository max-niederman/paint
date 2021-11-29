//! Caching and Querying Schemes for Canvas Resources

pub mod key;
mod resource;
pub mod view;

use key::*;
use view::*;

use std::time::SystemTime;

use canvas::{DateTime, Resource};
use miette::{IntoDiagnostic, Result, WrapErr};
use serde::{Deserialize, Serialize};
use sled::Tree;

pub trait Cache: Resource
where
    CacheEntry<Self>: Viewable,
{
    /// The key type represents both the structure of the cache and also how entries are duplicated.
    /// For example, using a [`CanvasPrefix`], [`CoursePrefix`], and [`IdPrefix`] would mean that independent entries are kept
    /// not only for each ID but also for each Canvas instance and each course thereof.
    type Key: Key;

    /// Merge or insert a resource into the cache.
    fn merge_insert(tree: &Tree, _viewer: &Viewer, key: &Self::Key, resource: Self) -> Result<()> {
        let new = CacheEntry {
            resource,
            updated: SystemTime::now().into(),
            last_accessed: None,
        };
        let entry = match Self::get_omniscient(tree, key)? {
            Some(old) => old.merge(&Viewer::Omniscient, new),
            None => new,
        };

        tree.insert(
            key.as_bytes(),
            bincode::serialize(&entry)
                .into_diagnostic()
                .wrap_err("while serializing resource")?,
        )
        .into_diagnostic()
        .wrap_err("while inserting into cache")?;

        Ok(())
    }

    /// Get a single resource from the cache.
    fn get<'t>(
        tree: &'t Tree,
        viewer: &Viewer,
        key: &Self::Key,
    ) -> Result<Option<View<CacheEntry<Self>>>> {
        let val = tree
            .get(key.as_bytes())
            .into_diagnostic()
            .wrap_err("while getting entry")?;

        val.map(|res| {
            bincode::deserialize(&res)
                .into_diagnostic()
                .wrap_err("while deserializing entry")
                .map(|e: CacheEntry<Self>| e.view(viewer).into_owned())
        })
        .transpose()
    }
    /// Get a single resource from the cache with omniscience.
    /// This is equivalent to calling [`get`] with [`Viewer::Omniscient`], except we return a raw [`CacheEntry`] rather than a [`View`] into one.
    fn get_omniscient(tree: &Tree, key: &Self::Key) -> Result<Option<CacheEntry<Self>>> {
        let val = tree
            .get(key.as_bytes())
            .into_diagnostic()
            .wrap_err("while get entry")?;

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
        viewer: &'v Viewer,
        prefix: &P,
    ) -> Box<dyn 'v + Iterator<Item = Result<(Self::Key, View<CacheEntry<Self>>)>>> {
        Box::new(tree.scan_prefix(prefix.as_bytes()).map(move |res| {
            let (key, val) = res.into_diagnostic()?;

            let key = Self::Key::parse_bytes(&mut key.iter().copied())?;

            let entry: CacheEntry<Self> = bincode::deserialize(val.as_ref())
                .into_diagnostic()
                .wrap_err("while deserializing entry")?;

            Ok((key, entry.view(viewer).into_owned()))
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<R> {
    resource: R,
    updated: DateTime,
    last_accessed: Option<DateTime>,
}
