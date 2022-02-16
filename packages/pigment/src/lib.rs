#![feature(async_closure)]
#![feature(never_type)]
#![feature(result_flattening)]
#![feature(generic_associated_types)]

extern crate canvas_lms as canvas;

pub mod cache;
pub mod error;
pub mod resource;
pub mod selector;
pub mod view;

pub use error::{Error, Result};
pub use resource::ResourceKind;
pub use selector::{DSelector, Selector};
pub use view::View;
