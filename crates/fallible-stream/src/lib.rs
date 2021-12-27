//! Utilities for manipulating fallible asynchronous streams.

#![feature(try_trait_v2)]
#![feature(fn_traits)]

pub mod extension;
pub mod flat_map;
pub mod yield_error;

pub use extension::FallibleStreamExt;
pub use flat_map::TryFlatMap;
pub use yield_error::YieldError;
