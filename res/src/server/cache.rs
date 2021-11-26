//! Caching and Querying Schemes for Canvas Resources

use std::{mem, time::SystemTime};

use canvas::{DateTime, Id, Resource};
use miette::{IntoDiagnostic, Result, WrapErr};
use serde::{Deserialize, Serialize};
use sled::Tree;

pub trait Cache: Resource {
    type Prefix: Prefix;

    /// Insert a resource into the cache.
    fn insert(
        tree: &Tree,
        canvas: &str,
        prefix: &Self::Prefix,
        id: &Id,
        resource: Self,
    ) -> Result<()> {
        let entry: CacheEntry<Self> = CacheEntry::<Self> {
            resource,
            inserted: SystemTime::now().into(),
            last_accessed: None,
        };
        tree.insert(
            Self::key(canvas, prefix, id),
            bincode::serialize(&entry)
                .into_diagnostic()
                .wrap_err("failed to serialize resource")?,
        )
        .into_diagnostic()
        .wrap_err("failed to insert into cache")?;
        Ok(())
    }

    /// Get a single resource from the cache.
    fn get(
        tree: &Tree,
        canvas: &str,
        prefix: &Self::Prefix,
        id: &Id,
    ) -> Result<Option<CacheEntry<Self>>> {
        let val = tree
            .get(Self::key(canvas, prefix, id))
            .into_diagnostic()
            .wrap_err("failed to get entry")?;

        val.map(|res| {
            bincode::deserialize(&res)
                .into_diagnostic()
                .wrap_err("failed to deserialize entry")
        })
        .transpose()
    }
    /// Get all resources matching the key from the cache.
    fn get_all(
        tree: &Tree,
        canvas: &str,
        prefix: &Self::Prefix,
    ) -> Box<dyn Iterator<Item = Result<(Id, CacheEntry<Self>)>>> {
        Box::new(
            tree.scan_prefix(Self::key_prefix(canvas, prefix))
                .map(|res| {
                    let (key, val) = res.into_diagnostic()?;
                    let id = Id::from_be_bytes(
                        key[key.len() - mem::size_of::<Id>()..].try_into().unwrap(),
                    );
                    let entry: CacheEntry<Self> = bincode::deserialize(val.as_ref())
                        .into_diagnostic()
                        .wrap_err("failed to deserialize entry")?;
                    Ok((id, entry))
                }),
        )
    }

    fn key_prefix(canvas: &str, prefix: &Self::Prefix) -> Vec<u8> {
        let mut key = Vec::with_capacity(canvas.len() + prefix.len() + 1);
        key.extend_from_slice(canvas.as_bytes());
        key.push(b'\0');
        key.extend_from_slice(prefix.as_bytes());
        key
    }

    fn key(canvas: &str, prefix: &Self::Prefix, id: &Id) -> Vec<u8> {
        let mut key = Vec::with_capacity(canvas.len() + prefix.len() + 1 + mem::size_of::<Id>());
        key.extend_from_slice(canvas.as_bytes());
        key.push(b'\0');
        key.extend_from_slice(prefix.as_bytes());
        key.extend_from_slice(&id.to_be_bytes());
        key
    }
}

pub trait Prefix {
    /// This type should be an array.
    fn as_bytes(&self) -> &[u8];
    fn len(&self) -> usize {
        self.as_bytes().len()
    }
}

pub struct NoPrefix;
impl Prefix for NoPrefix {
    fn as_bytes(&self) -> &[u8] {
        &[]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<R> {
    resource: R,
    inserted: DateTime,
    last_accessed: Option<DateTime>,
}

mod impls {
    use super::*;
    use canvas::resource::*;

    impl Cache for Course {
        type Prefix = NoPrefix;
    }
}
