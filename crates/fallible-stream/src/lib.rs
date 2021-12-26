//! Utilities for manipulating fallible asynchronous streams.

#![feature(try_trait_v2)]

mod yield_error;

pub use yield_error::YieldError;
