use super::prelude::*;
use canvas::resource::course::*;

// TODO: implement merge and remove [`UserKey`]

impl Cache for Course {
    type Key = CourseKey<UserKey<BaseKey>>;
}

impl Viewable for CacheEntry<Course> {
    fn merge(self, viewer: &Viewer, other: Self) -> Self {
        Self { ..other }
    }

    fn view(&self, viewer: &Viewer) -> View<Cow<'_, Self>> {
        match viewer {
            Viewer::Omniscient => View::Full(Cow::Borrowed(self)),
            Viewer::User(id) => View::Partial(Cow::Owned(Self {
                resource: Course {
                    enrollments: self
                        .resource
                        .enrollments
                        .iter()
                        .filter(|e| e.user_id == *id)
                        .cloned()
                        .collect(),
                    ..self.resource.clone()
                },
                ..self.clone()
            })),
        }
    }
}
