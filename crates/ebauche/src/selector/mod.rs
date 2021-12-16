//! Composable selectors by which queries are defined and efficiently executed.

use canvas::Resource;

pub mod trivial;
pub use trivial::*;

pub mod discriminated;
pub use discriminated::DSelector;

/// A [`Selector`] is a type which selects resources.
///
/// Selectors can be composed by parametrizing selector implementations. E.g., the [`And`] selector.
pub trait Selector<R: Resource> {
    /// Test if the selector matches a given [`Resource`].
    fn matches(&self, resource: &R) -> bool;
}
