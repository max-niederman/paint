use super::{shared::*, submission::Submission, Resource};
use crate::{DateTime, Id};
use chrono::Date;
use serde::{Deserialize, Serialize};

/// A Canvas Assignment.
///
/// Refer to [Canvas's API documentation](https://canvas.instructure.com/doc/api/assignments.html).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    pub id: Id,

    pub name: String,
    pub description: String,

    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub due_at: DateTime,
    pub lock_at: DateTime,
    pub unlock_at: DateTime,

    pub has_overrides: bool,
    pub all_dates: Option<Vec<serde_json::Value>>,
    pub course_id: Id,
    pub html_url: String,
    pub submissions_download_url: String,
    pub assignment_group_id: Id,
    pub due_date_required: bool,
    pub allowed_extensions: Vec<String>,
    pub max_name_length: u32,

    #[serde(default)]
    pub turnitin_enabled: Option<bool>,
    #[serde(default)]
    pub turnitin_settings: Option<TurnitinSettings>,

    #[serde(default)]
    pub vericite_enabled: Option<bool>,

    pub grade_group_students_individually: bool,
    pub external_tool_tag_attributes: Option<serde_json::Value>,
    pub peer_reviews: bool,

    pub automatic_peer_reviews: bool,
    #[serde(default)]
    pub peer_review_count: Option<u32>,
    #[serde(default)]
    pub peer_reviews_assign_at: Option<DateTime>,
    pub intra_group_peer_reviews: bool,

    pub group_category_id: Id,
    pub needs_grading_count: IfGradingRights<u32>,
    pub needs_grading_count_by_section: IfGradingRights<Vec<NeedsGradingCountBySection>>,

    pub position: u32,

    #[serde(default)]
    pub post_to_sis: Option<bool>,
    #[serde(default)]
    pub integration_id: Option<bool>,
    #[serde(default)]
    pub integration_data: Option<serde_json::Value>,

    pub points_possible: f64,
    pub submission_types: Vec<SubmissionType>,
    pub has_submitted_submissions: bool, // TODO: Show in web UI (afaik it isn't in the official UI)
    pub grading_type: GradingType,
    pub grading_standard_id: Option<Id>,

    pub published: bool,
    pub unpublishable: bool,
    pub only_visible_to_overrides: bool,
    pub locked_for_user: bool,
    #[serde(default)]
    pub lock_info: Option<LockInfo>,
    #[serde(default)]
    pub lock_explanation: Option<String>,

    pub quiz_id: Option<Id>,
    pub anonymous_submissions: Option<bool>,
    pub discussion_topic: Option<Id>,

    #[serde(default)]
    pub freeze_on_copy: Option<bool>,
    #[serde(default)]
    pub frozen: Option<bool>,
    #[serde(default)]
    pub frozen_attributes: Option<Vec<String>>,

    // present on include[]=submission
    #[serde(default)]
    pub submission: Option<Submission>,

    pub use_rubric_for_grading: Option<bool>,
    pub rubric_settings: Option<RubricSettings>,
    pub rubric: Option<Vec<RubricCriterion>>,

    // present on include[]=assignment_visibility
    #[serde(default)]
    pub assignment_visibility: Option<Vec<Id>>,

    // present on include[]=overrides
    #[serde(default)]
    pub overrides: Option<Vec<AssignmentOverride>>,

    pub omit_from_final_grade: Option<bool>,
    pub moderated_grading: bool,
    pub grader_count: u32,
    pub final_grader_id: Id,
    pub grader_comments_visible_to_graders: bool,
    pub graders_anonymous_to_graders: bool,
    pub grader_names_visible_to_final_grader: bool,
    pub anonymous_grading: bool,
    pub allowed_attempts: i32, // -1 is unlimited
    pub post_manually: bool,
    pub score_statistics: Option<ScoreStatistics>, // present on include[]=submission,score_statistics
    pub can_submit: bool, // present when retrieving single assignments and include[]=can_submit
    pub annotatable_attachment_id: Option<Id>,
}

impl Resource for Assignment {
    fn id(&self) -> Id {
        self.id
    }
}

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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GradingType {
    PassFail,
    Percent,
    LetterGrade,
    GpaScale,
    Points,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreStatistics {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
}

// FIXME: implement fields
type RubricSettings = serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RubricCriterion {
    pub points: u32,
    pub id: String,
    pub learning_outcome_id: Option<String>,
    pub vendor_guid: Option<String>,
    pub description: String,
    pub long_description: String,
    pub criterion_use_range: bool,
    pub ratings: Vec<RubricRating>,
    pub ignore_for_scoring: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RubricRating {
    pub points: u32,
    pub id: String,
    pub description: String,
    pub long_description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockInfo {
    pub asset_string: String,
    pub unlock_at: Option<DateTime>,
    pub lock_at: Option<DateTime>,
    pub context_module: String,
    pub manually_locked: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TurnitinSettings {
    pub originality_report_visibility: String,
    pub s_paper_check: bool,
    pub internet_check: bool,
    pub journal_check: bool,
    pub exclude_biblio: bool,
    pub exclude_quoted: bool,
    pub exclude_small_matches_type: String,
    pub exclude_small_matches_value: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeedsGradingCountBySection {
    pub section_id: String,
    pub needs_grading_count: u32,
}
