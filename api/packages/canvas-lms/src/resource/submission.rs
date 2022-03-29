use super::{Assignment, Course};
use crate::Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A Canvas Submission.
///
/// Refer to [Canvas's API documentation](https://canvas.instructure.com/doc/api/submissions.html).
#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Submission {
    pub course: Option<Course>,
    pub assignment_id: Id,
    pub assignment: Option<Assignment>,
    pub user_id: Id,
    pub attempt: Option<u32>,

    #[serde(default)]
    pub html_url: Option<String>,
    #[serde(default)]
    pub preview: Option<String>,

    pub posted_at: Option<DateTime<Utc>>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub graded_at: Option<DateTime<Utc>>,

    pub late: bool,
    pub excused: Option<bool>,
    pub missing: bool,

    pub late_policy_status: Option<LatePolicyStatus>,
    pub points_deducted: Option<f64>,
    pub seconds_late: Option<f64>,

    pub workflow_state: SubmissionWorkflowState,
    pub extra_attempts: Option<u32>,

    pub submission_type: Option<SubmissionType>,
    pub body: Option<String>,
    pub url: Option<String>,

    pub grade: Option<String>,
    pub score: Option<f64>,
}

#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionType {
    DiscussionTopic,
    OnlineQuiz,
    OnPaper,
    None,
    ExternalTool,
    OnlineTextEntry,
    OnlineUrl,
    OnlineUpload,
    MediaRecording,
    StudentAnnotation,
    BasicLtiLaunch,
    NotGraded,
}

#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionWorkflowState {
    Graded,
    Submitted,
    Unsubmitted,
    PendingReview,
}

#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LatePolicyStatus {
    Late,
    Missing,
    None,
}
