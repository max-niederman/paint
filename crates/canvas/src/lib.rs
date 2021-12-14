#![feature(result_flattening)]

#[cfg(feature = "client")]
pub mod client;
pub mod resource;

#[cfg(feature = "client")]
pub use client::{Auth, Client};
pub use resource::Resource;

pub type Id = u64;
pub type DateTime = chrono::DateTime<chrono::Utc>;
