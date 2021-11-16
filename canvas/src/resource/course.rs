use super::{enrollment::Enrollment, grading_period::GradingPeriod, shared::*, Resource};
use crate::{DateTime, Id};
use chrono::Date;
use serde::{Deserialize, Serialize};

/// A Canvas Course.
///
/// Refer to [Canvas's API documentation](https://canvas.instructure.com/doc/api/courses.html).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Course {
    pub id: Id,
    #[serde(default)]
    pub sis_course_id: Option<String>,
    pub uuid: String,
    #[serde(default)]
    pub integration_id: Option<Id>,
    #[serde(default)]
    pub sis_import_id: Option<Id>,

    pub name: String,
    pub course_code: String,
    pub workflow_state: WorkflowState,

    pub locale: String,
    pub time_zone: String,

    pub account_id: Id,
    pub root_account_id: Id,
    pub enrollment_term_id: Id,

    #[serde(default)]
    pub grading_periods: Vec<GradingPeriod>,
    pub grading_standard_id: Id,
    pub grade_passback_setting: String,

    pub created_at: DateTime,
    pub start_at: DateTime,
    pub end_at: DateTime,

    #[serde(default)]
    pub enrollments: Vec<Enrollment>, // present on include[]=total_scores
    pub total_students: Option<u32>,

    // TODO: implement Calendar API
    #[serde(default)]
    pub calendar: Option<serde_json::Value>,

    pub default_view: CourseView,
    pub syllabus_body: String,

    #[serde(default)]
    pub needs_grading_count: IfGradingRights<u32>, // present on include[]=needs_grading_count
    pub term: Option<Term>, // present on include[]=term
    pub course_progress: Option<CourseProgress>,
    pub apply_assignment_group_weights: bool,

    pub permissions: Option<Permissions>, // present when retrieving single assignments and include[]=permissions
    pub is_public: bool,
    pub is_public_to_auth_users: bool,
    pub public_syllabus: bool,
    pub public_syllabus_to_auth: bool,
    pub public_description: String,

    pub storage_quota_mb: f64,
    pub storage_quota_mb_used: f64,

    pub hide_final_grades: bool,
    pub license: String,

    pub allow_student_assignment_edits: bool,
    pub allow_wiki_comments: bool,
    pub allow_student_forum_attachments: bool,

    pub open_enrollment: bool,
    pub self_enrollment: bool,
    pub restrict_enrollments_to_course_dates: bool,

    pub course_format: String,
    #[serde(default)]
    pub access_restricted_by_date: Option<bool>,

    #[serde(default)]
    pub blueprint: Option<bool>,
    #[serde(default)]
    pub blueprint_restrictions: Option<serde_json::Value>,
    #[serde(default)]
    pub blueprint_restrictions_by_object_type: Option<serde_json::Value>,

    #[serde(default)]
    pub template: Option<bool>,
}

impl Resource for Course {
    fn id(&self) -> Id {
        self.id
    }
}

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
    pub start_at: DateTime,
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
