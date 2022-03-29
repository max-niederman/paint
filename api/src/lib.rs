#![feature(never_type)]
#![feature(once_cell)]
#![feature(box_patterns)]
#![feature(result_option_inspect)]
#![feature(trivial_bounds)]
#![feature(associated_type_defaults)]

pub mod auth;
pub mod error;
pub mod routes;
pub mod view;

pub use error::{Error, Result};

pub type HttpClient = hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>;
