use crate::Id;
use serde::{Deserialize, Serialize};

/// Wrapper type for values present only if we have grading rights.
pub type IfGradingRights<T> = Option<T>;

// FIXME: figure out how attachments are structured
pub type Attachment = serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Author {
    pub id: Option<Id>,
    pub display_name: Option<String>,
    pub avatar_image_url: Option<String>,
    pub html_url: Option<String>,
    pub pronouns: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Permissions {
    pub attach: bool,
    pub update: bool,
    pub reply: bool,
    pub delete: bool,
}
