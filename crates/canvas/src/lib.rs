#![feature(result_flattening)]

pub mod resource;
#[cfg(feature = "client")]
pub mod client;

pub use resource::Resource;
#[cfg(feature = "client")]
pub use client::Client;


pub type Id = u64;
pub type DateTime = chrono::DateTime<chrono::Utc>;
