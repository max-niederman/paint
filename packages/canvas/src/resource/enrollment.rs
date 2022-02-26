use super::user::User;
use crate::Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A Canvas Enrollment.
///
/// Refer to [Canvas's API documentation](https://canvas.instructure.com/doc/api/enrollments.html)
/// and [Source Code](https://github.com/instructure/canvas-lms/blob/master/app/controllers/enrollments_api_controller.rb).
#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Enrollment {
    #[serde(default)]
    pub id: Option<Id>,
    pub course_id: Id,
    pub course_section_id: Id,

    pub enrollment_state: EnrollmentState,
    #[serde(rename = "type")]
    pub enrollment_type: EnrollmentType,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub last_attended_at: DateTime<Utc>,
    pub total_activity_time: f64,

    pub html_url: String,
    pub grades: Grade, // WHY IS IT `grades` NOT `grade` AAAAAAAAAAAAAAAAAAAAAAAAAA
    pub user: User,

    pub user_id: Id,
    pub associated_user_id: Option<Id>, // set if we are enrolled as an observer
    pub role: EnrollmentRole,
    pub role_id: Id,
}

/// An inline enrollment. This includes all fields which are present when Enrollments are inlined in Courses.
#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InlineEnrollment {
    pub enrollment_state: EnrollmentState,
    #[serde(rename = "type")]
    pub enrollment_type: EnrollmentType,

    pub user_id: Id,
    pub associated_user_id: Option<Id>, // set if we are enrolled as an observer
    pub role: EnrollmentRole,
    pub role_id: Id,

    #[serde(default)]
    pub computed_current_score: Option<f64>,
    #[serde(default)]
    pub computed_final_score: Option<f64>,
    #[serde(default)]
    pub computed_current_grade: Option<String>,
    #[serde(default)]
    pub computed_final_grade: Option<String>,
}

#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentState {
    Active,
    Invited,
    Inactive,
}

#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentType {
    Student,
    Teacher,
    Ta,
    Designer,
    Observer,
}

#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EnrollmentRole {
    StudentEnrollment,
    TeacherEnrollment,
    TaEnrollment,
    DesignerEnrollment,
    ObserverEnrollment,
}

#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
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
}
