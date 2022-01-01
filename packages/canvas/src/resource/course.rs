use super::{enrollment::InlineEnrollment, grading_period::GradingPeriod, Resource};
use crate::{DateTime, Id};
use serde::{Deserialize, Serialize};

/// A Canvas Course.
///
/// Refer to [Canvas's API documentation](https://canvas.instructure.com/doc/api/courses.html).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Course {
    pub id: Id,
    pub uuid: String,

    pub name: String,
    pub course_code: String,
    pub workflow_state: WorkflowState,

    pub account_id: Id,
    pub enrollment_term_id: Id,

    #[serde(default)]
    pub grading_periods: Vec<GradingPeriod>,
    pub grading_standard_id: Option<Id>,

    pub created_at: DateTime,
    pub start_at: DateTime,
    pub end_at: Option<DateTime>,

    #[serde(default)]
    pub enrollments: Vec<InlineEnrollment>, // enrollment grades present on include[]=total_scores
    pub total_students: Option<u32>,

    pub default_view: CourseView,
    #[serde(default)]
    pub syllabus_body: Option<String>, // present on include[]=syllabus_body

    #[serde(default)]
    pub term: Option<Term>, // present on include[]=term
    pub course_progress: Option<CourseProgress>,

    pub permissions: Option<Permissions>, // present when retrieving single courses and include[]=permissions

    pub hide_final_grades: bool,

    #[serde(default)]
    pub course_format: Option<CourseFormat>, // present on include[]=course_format
    #[serde(default)]
    pub access_restricted_by_date: Option<bool>,
}

impl Resource for Course {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    Unpublished,
    Available,
    Completed,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CourseView {
    Feed,
    Wiki,
    Modules,
    Assignments,
    Syllabus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Term {
    pub id: Id,
    pub name: String,
    #[serde(default)]
    pub start_at: Option<DateTime>,
    #[serde(default)]
    pub end_at: Option<DateTime>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CourseProgress {
    pub requirement_count: u32,
    pub requirement_count_completed_count: u32,
    pub next_requirement_url: Option<String>,
    pub completed_at: Option<DateTime>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CourseFormat {
    OnCampus,
    Online,
    Blended,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Permissions {
    pub attach: bool,
    pub update: bool,
    pub reply: bool,
    pub delete: bool,
}
