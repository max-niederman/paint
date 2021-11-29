use super::prelude::*;
use canvas::resource::submission::*;

impl Cache for Submission {
    type Key = IdKey<CourseKey<UserKey<BaseKey>>>;
}

impl Viewable for CacheEntry<Submission> {
    fn view(&self, viewer: &Viewer) -> View<Cow<'_, Self>> {
        match viewer {
            Viewer::Omniscient => View::Full(Cow::Borrowed(self)),
            Viewer::User(id) if *id == self.resource.user_id => View::Full(Cow::Borrowed(self)),
            _ => View::None,
        }
    }
}
