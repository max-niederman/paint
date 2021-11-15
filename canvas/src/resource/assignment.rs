use super::shared::{Attachment, Author, Permissions};
use crate::{DateTime, Id};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    pub id: Id,
    pub description: String,
    pub due_at: Option<DateTime>,
    pub unlock_at: Option<DateTime>,
    pub lock_at: Option<DateTime>,
    pub points_possible: u32,
    pub grading_type: String,
    pub assignment_group_id: Id,
    pub grading_standard_id: Value,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub peer_reviews: bool,
    pub automatic_peer_reviews: bool,
    pub position: u32,
    pub grade_group_students_individually: bool,
    pub anonymous_peer_reviews: bool,
    pub group_category_id: Option<Id>,
    pub post_to_sis: bool,
    pub moderated_grading: bool,
    pub omit_from_final_grade: bool,
    pub intra_group_peer_reviews: bool,
    pub anonymous_instructor_annotations: bool,
    pub anonymous_grading: bool,
    pub graders_anonymous_to_graders: bool,
    pub grader_count: u32,
    pub grader_comments_visible_to_graders: bool,
    pub final_grader_id: Id,
    pub grader_names_visible_to_final_grader: bool,
    pub allowed_attempts: u32,
    pub annotatable_attachment_id: Value,
    pub secure_params: String,
    pub course_id: Id,
    pub name: String,
    pub submission_types: Vec<SubmissionType>,
    pub has_submitted_submissions: bool,
    pub due_date_required: bool,
    pub max_name_length: u32,
    pub in_closed_grading_period: bool,
    pub is_quiz_assignment: bool,
    pub can_duplicate: bool,
    pub original_course_id: Option<Id>,
    pub original_assignment_id: Option<Id>,
    pub original_assignment_name: Option<String>,
    pub original_quiz_id: Option<Id>,
    pub workflow_state: String,
    pub important_dates: bool,
    pub muted: bool,
    pub html_url: String,
    pub published: bool,
    pub only_visible_to_overrides: bool,
    pub locked_for_user: bool,
    pub submissions_download_url: String,
    pub post_manually: bool,
    pub anonymize_students: bool,
    pub require_lockdown_browser: bool,
    pub lock_info: Option<LockInfo>,
    pub use_rubric_for_grading: Option<bool>,
    pub free_form_criterion_comments: Option<bool>,
    #[serde(default)]
    pub rubric: Vec<RubricCriteria>,
    pub rubric_settings: Option<RubricSettings>,
    pub lock_explanation: Option<String>,
    pub discussion_topic: Option<DiscussionTopic>,
    pub peer_review_count: Option<u32>,
    pub peer_reviews_assign_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionType {
    None,
    OnPaper,
    ExternalTool,
    OnlineQuiz,
    OnlineTextEntry,
    OnlineUrl,
    OnlineUpload,
    MediaRecording,
    StudentAnnotation,
    DiscussionTopic,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockInfo {
    pub lock_at: DateTime,
    pub can_view: bool,
    pub asset_string: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RubricCriteria {
    pub id: Id,
    pub points: u32,
    pub description: String,
    pub long_description: String,
    pub criterion_use_range: bool,
    #[serde(default)]
    pub ratings: Vec<Rating>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RubricSettings {
    pub id: Id,
    pub title: String,
    pub points_possible: u32,
    pub free_form_criterion_comments: bool,
    pub hide_score_total: bool,
    pub hide_points: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rating {
    pub id: Id,
    pub points: f64,
    pub description: String,
    pub long_description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscussionTopic {
    pub assignment_id: Id,
    pub id: Id,
    pub title: String,
    pub last_reply_at: DateTime,
    pub created_at: DateTime,
    pub delayed_post_at: Option<DateTime>,
    pub posted_at: DateTime,
    pub root_topic_id: DateTime,
    pub position: Option<u32>,
    pub podcast_has_student_posts: bool,
    pub discussion_type: String,
    pub lock_at: Option<DateTime>,
    pub allow_rating: bool,
    pub only_graders_can_rate: bool,
    pub sort_by_rating: bool,
    pub is_section_specific: bool,
    pub user_name: Option<String>,
    pub discussion_subentry_count: u32,
    pub permissions: Permissions,
    pub require_initial_post: Value,
    pub user_can_see_posts: bool,
    pub podcast_url: Option<String>,
    pub read_state: String,
    pub unread_count: u32,
    pub subscribed: bool,
    pub attachments: Vec<Attachment>,
    pub published: bool,
    pub can_unpublish: bool,
    pub locked: bool,
    pub can_lock: bool,
    pub comments_disabled: bool,
    pub author: Author,
    pub html_url: String,
    pub url: String,
    pub pinned: bool,
    pub group_category_id: Id,
    pub can_group: bool,
    pub topic_children: Vec<Id>,
    pub group_topic_children: Vec<GroupTopicChildren>,
    pub locked_for_user: bool,
    pub lock_info: LockInfo,
    pub lock_explanation: String,
    pub message: String,
    pub todo_date: Option<DateTime>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupTopicChildren {
    pub id: Id,
    pub group_id: Id,
}

impl super::Resource for Assignment {
    fn id(&self) -> Id {
        self.id
    }
}
