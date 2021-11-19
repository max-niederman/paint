//! Fetchers for Canvas Resources

use std::{any::TypeId, collections::HashMap};

use canvas_lms::Resource;
use pigment::Selector;

/// The implementation of [`Fetch`] is responsible for fetching, deserializing, and temporarily storing the [`Resource`] from Canvas.
#[async_trait::async_trait]
pub trait Fetch<R, Resource, S: Selector<R>> {
    /// Fetch and store a superset of the resources matching the given [`Selector`].
    /// Takes an immutable reference to [`Self`] to allow for concurrent fetches.
    async fn fetch_superset(&self);
}

/// Implements logic for fetching multiple resource types from Canvas.
pub struct Fetcher {
    resources: HashMap<TypeId, RwLock<HashSet<Box<dyn Resource>>>>,
}

impl Fetcher {
    pub fn get_or_fetch<R: Resource, S: Selector<R>>(&self) where Self: Fetch<R, S> {}
}

/// A [`Selector`] for resources which might be fetch dependencies of the resources matching a given selector.
pub struct MightProduce<R: Resource, S: Selector<T>>;

/// Blanket implementation.
impl<R: Resource, T: Resource, S: Selector> Selector<R> for MightProduce<T, S> {
    fn matches(&self, resource: &R) -> bool {
        true
    }
}
