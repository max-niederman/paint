#![feature(result_flattening)]

pub mod client;
pub mod error;
pub mod resource;

pub use client::{Auth, Client};
pub use error::Error;
pub use resource::Resource;

pub type Result<T> = std::result::Result<T, Error>;

pub type Id = u64;
pub type DateTime = chrono::DateTime<chrono::Utc>;
