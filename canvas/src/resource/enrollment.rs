use super::{user::User, Resource};
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
    pub course_section_id: Id,

    pub enrollment_state: EnrollmentState,
    #[serde(rename = "type")]
    pub enrollment_type: EnrollmentType,

    pub user_id: Id,
    pub associated_user_id: Option<Id>, // set if we are enrolled as an observer
    pub role: EnrollmentState,
    pub role_id: Id,

    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub start_at: DateTime,
    pub end_at: DateTime,
    pub last_activity_at: DateTime,
    pub last_attended_at: DateTime,
    pub total_activity_time: f64,

    pub html_url: String,
    pub grades: Grade, // WHY IS IT `grades` NOT `grade` AAAAAAAAAAAAAAAAAAAAAAAAAA
    pub user: User,
}

impl Resource for Enrollment {}

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
}
