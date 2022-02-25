use serde::{de::DeserializeOwned, Serialize};

pub mod assignment;
pub mod course;
pub mod enrollment;
pub mod grading_period;
pub mod submission;
pub mod user;

pub use assignment::Assignment;
pub use course::Course;
pub use enrollment::{Enrollment, Grade};
pub use grading_period::GradingPeriod;
pub use submission::Submission;
pub use user::User;
