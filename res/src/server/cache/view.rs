use std::borrow::Cow;
use pigment::Viewer;

/// Behavior common to resources which can be constructed from views and into which views can be recreated.
pub trait Viewable: Sized + Clone {
    /// Merge this view into the underlying resource with another one.
    /// The default implementation is to return `other`. This implies that the views cannot be merged.
    fn merge(self, _viewer: &Viewer, other: Self) -> Self {
        other
    }

    /// Reconstruct a view into the resource from the merged one.
    fn view(&self, viewer: &Viewer) -> View<Cow<'_, Self>>;
}

/// A view into a resource.
/// This is basically a [`Cow`] but with additional semantics.
pub enum View<R> {
    Full(R),
    Partial(R),
    None,
}

impl<'u, R: Clone> View<Cow<'u, R>> {
    pub fn into_cow(self) -> Option<Cow<'u, R>> {
        match self {
            View::Full(r) => Some(r),
            View::Partial(r) => Some(r),
            View::None => None,
        }
    }

    pub fn into_owned(self) -> View<R> {
        match self {
            View::Full(r) => View::Full(r.into_owned()),
            View::Partial(r) => View::Partial(r.into_owned()),
            View::None => View::None,
        }
    }
}
