use super::shared::{Attachment, Author, Permissions};
use crate::{DateTime, Id};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Announcement {
    pub id: Id,
    pub root_topic_id: Option<Id>,
    pub is_section_specific: bool,
    pub delayed_post_at: DateTime,
    pub title: String,
    pub last_reply_at: DateTime,
    pub created_at: DateTime,
    pub posted_at: DateTime,
    pub assignment_id: Option<Id>,
    pub position: u32,
    pub podcast_has_student_posts: bool,
    pub discussion_type: String,
    pub lock_at: Option<DateTime>,
    pub allow_rating: bool,
    pub only_graders_can_rate: bool,
    pub sort_by_rating: bool,
    pub user_name: String,
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
    pub group_category_id: Option<Id>,
    pub can_group: bool,
    pub topic_children: Vec<Child>,
    pub group_topic_children: Vec<Child>,
    pub context_code: String,
    pub locked_for_user: bool,
    pub lock_info: LockInfo,
    pub lock_explanation: String,
    pub message: String,
    pub subscription_hold: String,
    pub todo_date: Option<DateTime>,
}

// FIXME: figure out how these keys are structured
pub type Child = Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockInfo {
    pub can_view: bool,
    pub asset_string: String,
}

impl super::Resource for Announcement {
    fn id(&self) -> Id {
        self.id
    }
}
