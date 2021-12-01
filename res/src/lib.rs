#![feature(min_specialization)]
#![feature(never_type)]

extern crate canvas_lms as canvas;

pub mod error;
pub mod message;
pub mod rpc;
pub mod selector;

use serde::{Deserialize, Serialize};

pub use error::{Error, Result};
pub use selector::{DSelector, Selector};

/// A viewer with respect to a resource. This loosely corresponds to an end user.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Viewer {
    User(canvas::Id),
    Omniscient,
}
