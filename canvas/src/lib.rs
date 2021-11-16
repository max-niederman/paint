pub mod http;
pub mod resource;

pub use http::Client;
pub use resource::Resource;

pub type Id = u64;
pub type DateTime = chrono::DateTime<chrono::Utc>;
