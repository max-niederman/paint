#![feature(result_flattening)]
#![feature(maybe_uninit_extra)]
#![feature(try_trait_v2)]

#[cfg(feature = "client")]
pub mod client;
pub mod id;
pub mod resource;

#[cfg(feature = "client")]
pub use client::{Auth, Client};
pub use id::Id;
pub use resource::Resource;

pub type DateTime = chrono::DateTime<chrono::Utc>;
