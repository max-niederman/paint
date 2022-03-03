//! Canvas resource fetching and caching.
//!
//! ## Resources and Collections
//!
//! Canvas **resources** are single objects returned from the Canvas API and defined in the `canvas_lms::resource` module. **Collections** are
//! homogeneous collections of one or more **resources**. All **resources** are also **collections** consisting of just one **resource**.
//!
//! ## Caching
//!
//! All **collections** can be cached in a simple key-value store (we currently use [`sled`]). To implement this, the [`Collection`] trait
//! has a method, [`Collection::cache_prefix`] which returns the tree name and key prefix which should contain the members of the **collection**.
//!
//! ## Fetching
//!
//! Most **collections** can be fetched from the Canvas API. This behavior is implemented using the [`Fetch`] trait, which is parameterized
//! over a client type, typically a [`canvas_lms::Client`] of some `hyper` connector.

mod impls;
pub mod cache;

use crate::view::View;
use futures::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

/// A **resource** contained in a view into a Canvas instance.
/// 
/// See module documentation for more details.
pub trait Resource
where
    Self: Serialize + DeserializeOwned,
{
    fn cache_location(&self, view: &View) -> CacheLocation;
}

impl<R: Resource> Collection for R {
    type Resource = Self;
    fn cache_prefix(&self, view: &View) -> CacheLocation {
        self.cache_location(view)
    }
}

/// A **collection** of one or more **resources**.
///
/// See module documentation for more details.
pub trait Collection {
    /// The type of **resource** of which the **collection** consists.
    type Resource: Resource;

    /// Get the cache prefix of the collection.
    fn cache_prefix(&self, view: &View) -> CacheLocation;
}

/// A **collection** which may be fetched from a **Canvas** API.
///
/// See module documentation for more details.
pub trait Fetch<'f, C>: Collection + 'f {
    type Err;

    type FetchAllStream: Stream<Item = Result<Self::Resource, Self::Err>> + Unpin;
    /// Get an asynchronous stream yielding all of **resources** of the **collection**.
    fn fetch_all(&'f self, view: &'f View, client: C) -> Self::FetchAllStream;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheLocation {
    /// The name of the keyspace.
    pub space: &'static str,

    /// The key or prefix of keys in the keyspace.
    pub key: Vec<u8>,
}

/// Re-exported **collection** types.
pub mod collections {
    pub use super::impls::course::*;
}