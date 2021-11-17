//! Fetchers for Canvas Resources

use canvas_lms::Resource;
use pigment::Selector;

/// The implementation of [`Fetch`] is responsible for fetching and deserializing the [`Resource`] from Canvas
pub trait Fetch: Resource {
    type Dependency: Resource;

    /// Fetch a superset of the resources matching the given [`Selector`] from Canvas
    pub fn fetch_superset<S>(selector: S) -> Vec<Self>;
}

pub struct NoDependencies;
impl Resource for NoDependencies {
    fn id(&self) -> Id {
        0
    }
}

/// A [`Selector`] for resources which might be fetch dependencies of the resources matching a given selector.
pub struct MightProduce<R: Resource, S: Selector<T>>;

/// Blanket implementation.
impl<R: Resource, T: Resource, S: Selector> Selector<R> for MightProduce<T, S> {
    fn matches(&self, resource: &R) -> bool {
        true
    }
}
