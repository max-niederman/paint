use super::{misc::*, Resource};
use crate::{DateTime, Id};
use serde::{Deserialize, Serialize};

/// A Canvas User.
///
/// Refer to [Canvas's API documentation](https://canvas.instructure.com/doc/api/users.html).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    id: Id,
    login_id: String,

    name: String,
    last_name: String,
    first_name: String,
    short_name: String,
}

impl Resource for User {}
