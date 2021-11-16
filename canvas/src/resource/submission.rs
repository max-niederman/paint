use super::{shared::*, Resource};
use crate::{DateTime, Id};
use serde::{Deserialize, Serialize};

/// A Canvas Submission.
///
/// Refer to [Canvas's API documentation](https://canvas.instructure.com/doc/api/submissions.html).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Submission; // FIXME: implement fields

impl Resource for Submission {
    fn id(&self) -> Id {
        unimplemented!()
    }
}