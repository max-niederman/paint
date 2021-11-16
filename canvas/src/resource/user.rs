use super::{shared::*, Resource};
use crate::{DateTime, Id};
use serde::{Deserialize, Serialize};

/// A Canvas User.
///
/// Refer to [Canvas's API documentation](https://canvas.instructure.com/doc/api/users.html).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User; // FIXME: implement fields

impl Resource for User {
    fn id(&self) -> Id {
        unimplemented!()
    }
}