use super::{submission::SubmissionType, Resource};
use crate::{DateTime, Id};
use serde::{Deserialize, Serialize};

/// A Canvas Assignment.
///
/// Refer to [Canvas's API documentation](https://canvas.instructure.com/doc/api/assignments.html).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    pub id: Id,

    pub name: String,
    pub description: Option<String>,

    pub course_id: Id,
    pub html_url: String,

    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub due_at: Option<DateTime>,
    pub lock_at: Option<DateTime>,
    pub unlock_at: Option<DateTime>,

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub due_at: Option<DateTime>,
    #[serde(default)]
    pub all_day: Option<bool>,
    #[serde(default)]
    pub all_day_date: Option<DateTime>,
    #[serde(default)]
    pub unlock_at: Option<DateTime>,
    #[serde(default)]
    pub lock_at: Option<DateTime>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GradingType {
    PassFail,
    Percent,
    LetterGrade,
    GpaScale,
    Points,
    NotGraded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreStatistics {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockInfo {
    pub asset_string: String,
    pub unlock_at: Option<DateTime>,
    pub lock_at: Option<DateTime>,
}
