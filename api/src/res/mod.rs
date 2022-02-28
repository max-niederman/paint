//! Canvas resource fetching and caching.
//!
//! ## Resource Tree
//!
//! Imagine that all the resources of all possible views (of users into a Canvas instance) are arranged in a rooted
//! tree. The leaves of the tree represent API requests which return a single resource. Forks represent sets of resources,
//! sometimes with an associated API request. Branches represent containment of resources sets; e.g. the node representing
//! active Courses only would be a child of the node representing all Courses.
//!
//! ### Homogenous Nodes
//!
//! Leaves and usually their close ancestors are "homogenous," meaning that they represent only a single type of resources. More formally,
//! the set of resources is a subset of the set of values of a Rust type. This is represented by the trait [`HomoNode`] which is a
//! subtrait of [`Node`] and has an associated type [`HomoNode::Resource`].
//!
//! ## Fetch
//!
//! Nodes which correspond to API requests also implement [`FetchAll`] and/or [`FetchOne`]. They must implement a method which
//! takes `&self` and an HTTP client and returns a homogeneous set of resources. Typically, this is achieved by constructing a
//! URL based on the node's ancestors. Homogeneity is ensured by the supertrait bound `Fetch: HomoNode`.
//!
//! ## Cache
//!
//! Some [`Node`]s represent logical "prefixes" in the cache. This prefix, represented as a [`CachePrefix`] and optionally
//! returned by [`Node::cache_prefix`] contains a keyspace name and a possibly empty key prefix represented as a byte vector.
//!
//! ## HTTP Endpoints
//!
//! TODO: add poem endpoints

pub mod impls;

use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// A node in the resource tree.
///
/// See module documentation for more details.
pub trait Node {
    /// Get the cache prefix of the node, if there is one.
    fn cache_prefix(&self) -> Option<CachePrefix> {
        None
    }
}
/// A node in the resource tree representing a homogenous set of resources.
///
/// See module documentation for more details.
pub trait HomoNode<'r>: Node {
    type Resource: Serialize + Deserialize<'r> + 'r;
}

/// A node representing an API endpoint which can be fetched.
///
/// One of the two methods must be implemented, or both will recurse infinitely.
///
/// See module documentation for more details.
pub trait FetchAll<'r, C>
where
    Self: HomoNode<'r> + Sync,
    Self::Resource: Send,
    C: Sync,
{
    type Err;

    type FetchAllStream: Stream<Item = Result<Self::Resource, Self::Err>> + Send + 'r;
    fn fetch_all(&'r self, client: &'r C) -> Self::FetchAllStream;
}

pub trait FetchOne<'r, C>
where
    Self: HomoNode<'r> + Sync,
    Self::Resource: Send,
    C: Sync,
{
    type Err;

    type FetchOneFuture: Future<Output = Result<Self::Resource, Self::Err>> + Send + 'r;
    fn fetch_one(&'r self, client: &'r C) -> Self::FetchOneFuture;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachePrefix {
    /// The name of the keyspace.
    ///
    /// It's a [`&'static str`] to ensure [`Node`]s don't accidentally make way too many [`sled::Tree`]s.
    pub key_space: &'static str,

    /// The prefix of keys in the keyspace.
    pub key_prefix: Vec<u8>,
}

impl CachePrefix {
    /// Join the [`CachePrefix`] with a key prefix.
    pub fn join_key_prefix(mut self, key_prefix: &[u8]) -> Self {
        self.key_prefix.extend_from_slice(key_prefix);
        self
    }
}
