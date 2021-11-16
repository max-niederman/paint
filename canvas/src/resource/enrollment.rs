use super::{shared::*, user::User, Resource};
use crate::{DateTime, Id};
use serde::{Deserialize, Serialize};

/// A Canvas Enrollment.
///
/// Refer to [Canvas's API documentation](https://canvas.instructure.com/doc/api/enrollments.html)
/// and [Source Code](https://github.com/instructure/canvas-lms/blob/master/app/controllers/enrollments_api_controller.rb).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Enrollment {
    pub id: Id,
    pub course_id: Id,
    #[serde(default)]
    pub sis_course_id: Option<String>,
    #[serde(default)]
    pub course_integration_id: Option<String>,
    pub course_section_id: Id,
    #[serde(default)]
    pub section_integration_id: Option<String>,
    #[serde(default)]
    pub sis_account_id: Option<String>,
    #[serde(default)]
    pub sis_section_id: Option<String>,
    #[serde(default)]
    pub sis_user_id: Option<String>,
    #[serde(default)]
    pub sis_import_id: Option<Id>,

    pub enrollment_state: EnrollmentState,
    pub limit_privileges_to_course_section: bool,
    pub root_account_id: Id,

    #[serde(rename = "type")]
    pub enrollment_type: EnrollmentType,
    pub user_id: Id,
    pub associated_user_id: Option<Id>, // set if we are enrolled as an observer
    pub role: EnrollmentState,
    pub role_id: Id,

    pub csreated_at: DateTime,
    pub updated_at: DateTime,
    pub start_at: DateTime,
    pub end_at: DateTime,
    pub last_activity_at: DateTime,
    pub last_attended_at: DateTime,
    pub total_activity_time: f64,

    pub html_url: String,
    pub grades: Grade, // WHY IS IT `grades` NOT `grade` AAAAAAAAAAAAAAAAAAAAAAAAAA
    pub user: User,

    // these aren't documented as optional, but they don't appear in most users' enrollments
    #[serde(default)]
    pub override_grade: Option<String>,
    #[serde(default)]
    pub override_score: Option<f64>,

    #[serde(default)]
    pub unposted_current_grade: Option<String>,
    #[serde(default)]
    pub unposted_final_grade: Option<String>,
    #[serde(default)]
    pub unposted_current_score: Option<String>,
    #[serde(default)]
    pub unposted_final_score: Option<String>,

    #[serde(default)]
    pub has_grading_periods: Option<bool>,
    #[serde(default)]
    pub totals_for_all_grading_periods: Option<bool>,

    #[serde(default)]
    pub current_grading_period_title: Option<String>,
    #[serde(default)]
    pub current_grading_period_id: Option<Id>,
    #[serde(default)]
    pub current_period_override_grade: Option<String>,
    #[serde(default)]
    pub current_period_override_score: Option<f64>,
    #[serde(default)]
    pub current_period_unposted_current_score: Option<f64>,
    #[serde(default)]
    pub current_period_unposted_final_score: Option<f64>,
    #[serde(default)]
    pub current_period_unposted_current_grade: Option<String>,
    #[serde(default)]
    pub current_period_unposted_final_grade: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentState {
    Active,
    Invited,
    Inactive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EnrollmentType {
    StudentEnrollment,
    TeacherEnrollment,
    TaEnrollment,
    DesignerEnrollment,
    ObserverEnrollment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grade {
    pub html_url: String,

    #[serde(default)]
    pub current_grade: Option<String>,
    #[serde(default)]
    pub final_grade: Option<String>,
    #[serde(default)]
    pub current_score: Option<String>,
    #[serde(default)]
    pub final_score: Option<String>,
    #[serde(default)]
    pub current_points: Option<f64>, // present when include[]=current_points

    #[serde(default)]
    pub unposted_current_grade: Option<String>,
    #[serde(default)]
    pub unposted_final_grade: Option<String>,
    #[serde(default)]
    pub unposted_current_score: Option<String>,
    #[serde(default)]
    pub unposted_final_score: Option<String>,
    #[serde(default)]
    pub unposted_current_points: Option<f64>, // present when include[]=current_points
}
