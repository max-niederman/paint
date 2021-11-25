//! Fetchers for Canvas Resources

use async_trait::async_trait;
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    marker::PhantomData,
};
use tokio::sync::RwLock;

use canvas_lms::Resource;
use pigment::selector::{self, Selector};

mod impls;

/// The implementation of [`Fetch`] is responsible for fetching, deserializing, and temporarily storing the [`Resource`] from Canvas.
#[async_trait]
pub trait Fetch<R: Resource, S: Selector<R>> {
    /// Fetch and store a superset of the resources matching the given [`Selector`].
    ///
    /// Implementations of this method are also responsible for recursively fetching the dependencies of the fetched resources
    /// using their respective [`Fetch`] implementations. Therefore, implementations of this method should strive not to fetch
    /// a resource which is already in the [`Fetcher`].
    async fn fetch_superset(&self);
}

/// Implements logic for fetching multiple resource types from Canvas.
pub struct Fetcher {
    client: canvas_lms::Client,
    resources: RwLock<HashMap<TypeId, RwLock<HashSet<Box<dyn Any>>>>>,
}

impl Fetcher {
    /// Construct a new empty [`Fetcher`].
    pub fn new(client: canvas_lms::Client) -> Self {
        Self {
            client,
            resources: RwLock::new(HashMap::new()),
        }
    }

    pub fn client(&self) -> &canvas_lms::Client {
        &self.client
    }

    pub fn resources(&self) -> &RwLock<HashMap<TypeId, RwLock<HashSet<Box<dyn Any>>>>> {
        &self.resources
    }

    /// Ensure that the resource map is initialized for a given resource type.
    /// This is helpful for ensuring we don't hold onto a write guard for the outer lock longer than necessary.
    async fn ensure_resource_initialized<R>(&self)
    where
        R: Resource + 'static,
    {
        self.resources
            .write()
            .await
            .entry(TypeId::of::<R>())
            .or_default();
    }
}

/// A [`Selector`] for resources which might be fetch dependencies of the resources matching a given selector.
///
/// Can be read as "might produce a resource `T` such that `T` matches the selector `S`."
pub struct MightProduce<T: Resource, S: Selector<T>> {
    selector: S,
    _target: PhantomData<T>,
}

/// Blanket implementation.
impl<R, T, S> Selector<R> for MightProduce<T, S>
where
    R: Resource,
    T: Resource,
    S: Selector<T>,
{
    fn matches(&self, _resource: &R) -> bool {
        true
    }
}
