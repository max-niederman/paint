use super::prelude::*;
use canvas_lms::resource::course::*;

impl Resource for Course {
    fn cache_location(&self, view: &View) -> CacheLocation {
        CacheLocation {
            space: "course",
            key: [
                view.id.as_bytes().as_slice(),
                self.id.to_be_bytes().as_slice(),
            ]
            .concat(),
        }
    }
}


impl_collection_fetch!(Course, paginated, |_, _| "/api/v1/courses".into());

#[derive(Debug, Clone, Copy)]
pub struct CourseById(canvas_lms::Id);

impl Collection for CourseById {
    type Resource = Course;

    fn cache_prefix(&self, view: &View) -> CacheLocation {
        CacheLocation {
            space: "course",
            key: [
                view.id.as_bytes().as_slice(),
                self.0.to_be_bytes().as_slice(),
            ]
            .concat(),
        }
    }
}

impl_collection_fetch!(CourseById, single, |CourseById(id), _| format!("/api/v1/courses/{id}"));
