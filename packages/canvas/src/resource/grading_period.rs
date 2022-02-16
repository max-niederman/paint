use super::Resource;
use crate::Id;
use serde::{Deserialize, Serialize};

/// A Canvas Grading Period.
///
/// Refer to [Canvas's API documentation](https://canvas.instructure.com/doc/api/grading_periods.html).
#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradingPeriod {
    pub id: Id,
    pub title: String,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub close_date: chrono::DateTime<chrono::Utc>,
    pub weight: f64,
    pub is_closed: bool,
}

impl Resource for GradingPeriod {}
