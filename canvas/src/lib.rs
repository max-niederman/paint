pub mod resource;
pub mod http;

pub use resource::{Announcement, Assignment, Res, Resource};
pub use http::Client;

pub type Id = u64;
pub type DateTime = chrono::DateTime<chrono::Utc>;