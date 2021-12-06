#![feature(specialization)]
#![feature(generic_const_exprs)]
#![feature(marker_trait_attr)]
#![feature(never_type)]

extern crate canvas_lms as canvas;

pub mod cache;
pub mod error;
pub mod view;
pub mod rpc;
pub mod selector;

pub use error::{Error, Result};
pub use selector::{DSelector, Selector};
pub use view::View;
