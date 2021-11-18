use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Permissions {
    pub attach: bool,
    pub update: bool,
    pub reply: bool,
    pub delete: bool,
}
