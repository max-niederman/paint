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

/// A view into a Canvas instance.
///
/// At the moment, there are only [`User`] views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum View {
    User(canvas::Id),
}
