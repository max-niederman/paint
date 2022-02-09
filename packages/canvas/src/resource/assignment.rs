use super::{submission::SubmissionType, Resource};
use crate::Id;
use serde::{Deserialize, Serialize};

/// A Canvas Assignment.
///
/// Refer to [Canvas's API documentation](https://canvas.instructure.com/doc/api/assignments.html).
#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    pub id: Id,

    pub name: String,
    pub description: Option<String>,

    pub course_id: Id,
    pub html_url: String,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub due_at: Option<chrono::DateTime<chrono::Utc>>,
    pub lock_at: Option<chrono::DateTime<chrono::Utc>>,
    pub unlock_at: Option<chrono::DateTime<chrono::Utc>>,

    pub submission_types: Vec<SubmissionType>,
    pub has_submitted_submissions: bool,

    #[serde(default)]
    pub score_statistics: Option<ScoreStatistics>, // included on include[]=score_statistics,submission

    pub locked_for_user: bool,
    #[serde(default)]
    pub lock_info: Option<LockInfo>,

    pub grading_type: GradingType,
}

impl Resource for Assignment {}

#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssignmentOverride {
    pub id: Id,
    pub assignment_id: Id,
    pub title: String,

    #[serde(default)]
    pub student_ids: Option<Vec<Id>>,
    #[serde(default)]
    pub group_id: Option<Id>,
    #[serde(default)]
    pub course_section_id: Option<Id>,

    #[serde(default)]
    pub due_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub all_day: Option<bool>,
    #[serde(default)]
    pub all_day_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub unlock_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub lock_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GradingType {
    PassFail,
    Percent,
    LetterGrade,
    GpaScale,
    Points,
    NotGraded,
}

#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreStatistics {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
}

#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockInfo {
    pub asset_string: String,
    pub unlock_at: Option<chrono::DateTime<chrono::Utc>>,
    pub lock_at: Option<chrono::DateTime<chrono::Utc>>,
}
