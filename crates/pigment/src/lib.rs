#![feature(async_closure)]
#![feature(generic_associated_types)]
#![feature(never_type)]
#![feature(result_flattening)]

extern crate canvas_lms as canvas;

pub mod cache;
pub mod error;
pub mod selector;
pub mod view;

pub use error::{Error, Result};
pub use selector::{DSelector, Selector};
pub use view::View;
