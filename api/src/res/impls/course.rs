use super::prelude::*;
use canvas_lms::resource::course::*;

impl Resource for Course {
    fn query_string() -> &'static str {
        "include[]=course_progress&include[]=syllabus_body"
    }
}

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

impl_collection_fetch!(AllCourses, paginated, |_, _| "/api/v1/courses".into());

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

impl_collection_fetch!(CourseById, single, |CourseById(id), _| format!(
    "/api/v1/courses/{id}"
));
