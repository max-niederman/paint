use super::prelude::*;
use canvas_lms::resource::course::*;

impl Resource for Course {}

const STD_INCLUDE: &[&'static str] = &["course_progress", "syllabus_body", "term", "favorites"];

#[derive(Debug, Clone, Copy)]
pub struct AllCourses;

impl Collection for AllCourses {
    type Resource = Course;

    fn cache_prefix(&self, view: &View) -> CacheLocation {
        CacheLocation {
            space: "course",
            key: view.id.as_bytes().to_vec(),
        }
    }

    fn cache_location(&self, view: &View, resource: &Self::Resource) -> CacheLocation {
        CacheLocation {
            space: "course",
            key: [
                view.id.as_bytes().as_slice(),
                resource.id.to_be_bytes().as_slice(),
            ]
            .concat(),
        }
    }
}

impl_collection_fetch! {
    collection = AllCourses;
    method = PAGINATED;
    path = |_, _| "/api/v1/courses".into();
    include = STD_INCLUDE.iter().copied();
}

#[derive(Debug, Clone, Copy)]
pub struct CourseById(pub canvas_lms::Id);

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

    fn cache_location(&self, view: &View, resource: &Self::Resource) -> CacheLocation {
        CacheLocation {
            space: "course",
            key: [
                view.id.as_bytes().as_slice(),
                resource.id.to_be_bytes().as_slice(),
            ]
            .concat(),
        }
    }
}

impl_collection_fetch! {
    collection = CourseById;
    method = SINGLE;
    path = |CourseById(id), _| format!(
        "/api/v1/courses/{id}"
    );
    include = STD_INCLUDE.iter().copied();
}
