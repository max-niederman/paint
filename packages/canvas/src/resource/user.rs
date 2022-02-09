use super::Resource;
use crate::Id;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A Canvas User.
///
/// Refer to [Canvas's API documentation](https://canvas.instructure.com/doc/api/users.html).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct User {
    pub id: Id,
    pub login_id: String,

    pub name: String,
    pub last_name: String,
    pub first_name: String,
    pub short_name: String,
}

impl Resource for User {}
