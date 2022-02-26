use crate::Id;
use chrono::{DateTime, Utc};
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
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub close_date: DateTime<Utc>,
    pub weight: f64,
    pub is_closed: bool,
}
