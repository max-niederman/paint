#![feature(min_specialization)]

extern crate canvas_lms as canvas;

pub mod message;
pub mod selector;

use serde::{Serialize, Deserialize};

pub use selector::{Selector, DSelector};

/// A viewer with respect to a resource. This loosely corresponds to an end user.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Viewer {
    User(canvas::Id),
    Omniscient,
}
