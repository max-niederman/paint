#![feature(result_flattening)]
#![feature(try_trait_v2)]
#![feature(maybe_uninit_extra)]

#[cfg(feature = "client")]
pub mod client;
pub mod id;
pub mod resource;

#[cfg(feature = "client")]
pub use client::{Auth, Client};
pub use id::Id;
