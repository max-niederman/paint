use crate::Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A Canvas User.
///
/// Refer to [Canvas's API documentation](https://canvas.instructure.com/doc/api/users.html).
#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct User {
    pub id: Id,

    pub created_at: DateTime<Utc>,

    pub name: String,
    pub short_name: String,
    pub sortable_name: String,

    pub locale: Option<String>,
    pub effective_locale: Option<String>,
}
