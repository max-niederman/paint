use super::Resource;
use crate::{DateTime, Id};
use serde::{Deserialize, Serialize};

/// A Canvas Grading Period.
///
/// Refer to [Canvas's API documentation](https://canvas.instructure.com/doc/api/grading_periods.html).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradingPeriod {
    pub id: Id,
    pub title: String,
    pub start_date: DateTime,
    pub end_date: DateTime,
    pub close_date: DateTime,
    pub weight: f64,
    pub is_closed: bool,
}

impl Resource for GradingPeriod {}
