//! Composable selectors by which queries are defined and efficiently executed.

use canvas_lms::Id;
use canvas_lms::Resource;

pub mod trivial;
pub use trivial::*;

/// A [`Selector`] is a type which selects resources.
///
/// Selectors can be composed by parametrizing selector implementations. E.g., the [`And`] selector.
pub trait Selector<R: Resource> {
    /// Test if the selector matches a given [`Resource`].
    fn matches(&self, resource: &R) -> bool;
}
