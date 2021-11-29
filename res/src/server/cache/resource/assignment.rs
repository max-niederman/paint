use super::prelude::*;
use canvas::resource::assignment::*;

// TODO: test implementing merge and removing `UserKey`

impl Cache for Assignment {
    type Key = IdKey<CourseKey<UserKey<BaseKey>>>;
}

impl Viewable for CacheEntry<Assignment> {
    fn view(&self, viewer: &Viewer) -> View<Cow<'_, Self>> {
        match viewer {
            Viewer::Omniscient => View::Full(Cow::Borrowed(self)),
            // we can't actually know whether the viewer can view this in any other circumstance
            _ => View::Full(Cow::Borrowed(self)),
        }
    }
}
