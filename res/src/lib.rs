#![feature(min_specialization)]

extern crate canvas_lms as canvas;

pub mod selector;
pub use selector::{Selector, DSelector};

pub mod message;

use serde::{Serialize, Deserialize};

/// A viewer with respect to a resource. This loosely corresponds to an end user.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Viewer {
    User(canvas::Id),
    Omniscient,
}