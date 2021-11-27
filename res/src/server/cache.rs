//! Caching and Querying Schemes for Canvas Resources

use std::{borrow::Cow, ffi::CString, mem, time::SystemTime};

use canvas::{DateTime, Id, Resource};
use miette::{IntoDiagnostic, Result, WrapErr};
use serde::{Deserialize, Serialize};
use sled::Tree;

pub trait Cache: Resource {
    /// The prefix represents both the structure of the cache and also how entries are duplicated.
    /// For example, using a [`CanvasPrefix`] and [`CoursePrefix`] would mean that independent entries are kept
    /// not only for each ID but also for each Canvas instance and each course thereof.
    type KeyPrefix: Key;

    /// Merge this view into the underlying resource with another one.
    /// The default implementation is to return `other`. This implies that the views cannot be merged.
    fn merge(self, other: Self) -> Self {
        other
    }
    /// Reconstruct a view into the resource from the merged one.
    /// If the user has full access to the view, return `View::Full(self)`.
    /// This implies that the view was originally derived from the user's view and is not merged.
    fn view(&self, user: Id) -> View<'_, Self> {
        View::Full(self)
    }

    /// Merge or insert a resource into the cache.
    fn merge_insert(tree: &Tree, prefix: &Self::KeyPrefix, id: &Id, resource: Self) -> Result<()> {
        let entry = match Self::get(tree, prefix, id)? {
            Some(old_entry) => CacheEntry {
                resource: old_entry.resource.merge(resource),
                updated: SystemTime::now().into(),
                last_accessed: old_entry.last_accessed,
            },
            None => CacheEntry {
                resource,
                updated: SystemTime::now().into(),
                last_accessed: None,
            },
        };

        tree.insert(
            Self::key(prefix, id),
            bincode::serialize(&entry)
                .into_diagnostic()
                .wrap_err("failed to serialize resource")?,
        )
        .into_diagnostic()
        .wrap_err("failed to insert into cache")?;

        Ok(())
    }

    /// Get a single resource from the cache.
    fn get(tree: &Tree, prefix: &Self::KeyPrefix, id: &Id) -> Result<Option<CacheEntry<Self>>> {
        let val = tree
            .get(Self::key(prefix, id))
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
    fn get_all<P: KeyPrefix<Self::KeyPrefix>>(
        tree: &Tree,
        prefix: &P,
    ) -> Box<dyn Iterator<Item = Result<(Id, CacheEntry<Self>)>>> {
        Box::new(tree.scan_prefix(prefix.as_bytes()).map(|res| {
            let (key, val) = res.into_diagnostic()?;
            let id = Id::from_be_bytes(key[key.len() - mem::size_of::<Id>()..].try_into().unwrap());
            let entry: CacheEntry<Self> = bincode::deserialize(val.as_ref())
                .into_diagnostic()
                .wrap_err("failed to deserialize entry")?;
            Ok((id, entry))
        }))
    }

    fn key(prefix: &Self::KeyPrefix, id: &Id) -> Vec<u8> {
        IdKey { prefix, id: *id }.as_bytes().to_vec()
    }
}

/// A structured key in the cache [`Tree`].
pub trait Key {
    type Bytes: AsRef<[u8]>;
    fn as_bytes(&self) -> Self::Bytes;
}

pub trait KeyPrefix<K: Key>: Key {}
impl<K: Key> KeyPrefix<K> for K {}

impl<K: Key> Key for &K {
    type Bytes = K::Bytes;
    fn as_bytes(&self) -> Self::Bytes {
        (*self).as_bytes()
    }
}

impl Key for () {
    type Bytes = [u8; 0];
    fn as_bytes(&self) -> Self::Bytes {
        []
    }
}

pub struct CanvasKey<P> {
    pub prefix: P,
    // the instance must not contain a null byte, so we use [`CString`]
    pub instance: CString,
}
impl<P: Key> Key for CanvasKey<P> {
    type Bytes = Vec<u8>;
    fn as_bytes(&self) -> Self::Bytes {
        [
            self.prefix.as_bytes().as_ref(),
            &self.instance.as_bytes_with_nul(),
        ]
        .concat()
    }
}
impl<P: Key> KeyPrefix<CanvasKey<P>> for P {}

pub struct IdKey<P> {
    pub prefix: P,
    pub id: Id,
}
impl<'p, P: Key> Key for IdKey<P> {
    type Bytes = Vec<u8>;
    fn as_bytes(&self) -> Self::Bytes {
        [self.prefix.as_bytes().as_ref(), &self.id.to_be_bytes()].concat()
    }
}
impl<P: Key> KeyPrefix<IdKey<P>> for P {}

// we alias these to make the semantics clearer
pub type CourseKey<P> = IdKey<P>;
pub type UserKey<P> = IdKey<P>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<R> {
    resource: R,
    updated: DateTime,
    last_accessed: Option<DateTime>,
}

/// A view into a resource.
/// This is basically a [`Cow`] but with additional semantics.
pub enum View<'a, R> {
    Full(&'a R),
    Partial(R),
    // TODO: should there be a `None` variant representing a null view?
}
impl<'a, R> View<'a, R> {
    pub fn into_cow(self) -> Cow<'a, R>
    where
        R: Clone,
    {
        match self {
            View::Full(r) => Cow::Borrowed(r),
            View::Partial(r) => Cow::Owned(r),
        }
    }
}

mod impls {
    use super::*;
    use canvas::resource::*;

    type BaseKey = CanvasKey<()>;

    impl Cache for Assignment {
        // TODO: benchmark implementing merge and removing `UserKey`
        type KeyPrefix = CourseKey<UserKey<BaseKey>>;
    }
    impl Cache for Course {
        // TODO: implement merge and remove `UserKey`
        type KeyPrefix = UserKey<BaseKey>;
    }
    impl Cache for Enrollment {
        // TODO:
        type KeyPrefix = UserKey<BaseKey>;
    }
    impl Cache for GradingPeriod {
        type KeyPrefix = CourseKey<BaseKey>;
    }
    impl Cache for Submission {
        type KeyPrefix = CourseKey<UserKey<BaseKey>>;
    }
    impl Cache for User {
        type KeyPrefix = BaseKey;
    }
}
