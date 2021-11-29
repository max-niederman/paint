mod assignment;
mod course;
mod submission;

mod prelude {
    pub use crate::cache::{key::*, view::*, *};
    pub use std::borrow::Cow;

    pub type BaseKey = CanvasKey<()>;
}
use prelude::*;

default impl<R: Resource + Viewable> Viewable for CacheEntry<R> {
    fn merge(self, viewer: &Viewer, other: Self) -> Self {
        Self {
            resource: self.resource.merge(viewer, other.resource),
            last_accessed: other.last_accessed.or(self.last_accessed),
            updated: other.updated.max(self.updated),
        }
    }

    fn view(&self, viewer: &Viewer) -> View<Cow<'_, Self>> {
        match viewer {
            Viewer::Omniscient => View::Full(Cow::Borrowed(self)),

            _ => match self.resource.view(viewer).into_cow() {
                Some(resource) => View::Partial(Cow::Owned(Self {
                    resource: resource.into_owned(),
                    ..self.clone()
                })),
                None => View::None,
            },
        }
    }
}
