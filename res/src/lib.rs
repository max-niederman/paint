#![feature(marker_trait_attr)]
#![feature(never_type)]
#![feature(associated_type_defaults)]
#![feature(result_flattening)]

extern crate canvas_lms as canvas;

pub mod cache;
pub mod error;
pub mod rpc;
pub mod selector;
pub mod view;

pub use error::{Error, Result};
pub use selector::{DSelector, Selector};
pub use view::View;
