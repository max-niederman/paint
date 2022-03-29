use super::submission::SubmissionType;
use crate::Id;
use chrono::{DateTime, Utc};
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

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
    pub lock_at: Option<DateTime<Utc>>,
    pub unlock_at: Option<DateTime<Utc>>,

    pub submission_types: Vec<SubmissionType>,
    pub has_submitted_submissions: bool,

    #[serde(default)]
    pub score_statistics: Option<ScoreStatistics>, // included on include[]=score_statistics,submission

    pub locked_for_user: bool,
    #[serde(default)]
    pub lock_info: Option<LockInfo>,

    pub grading_type: GradingType,
}

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
    pub due_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub all_day: Option<bool>,
    #[serde(default)]
    pub all_day_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub unlock_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub lock_at: Option<DateTime<Utc>>,
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
    pub unlock_at: Option<DateTime<Utc>>,
    pub lock_at: Option<DateTime<Utc>>,
}
