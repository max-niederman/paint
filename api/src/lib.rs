#![feature(never_type)]
#![feature(once_cell)]
#![feature(box_patterns)]
#![feature(result_option_inspect)]
#![feature(trivial_bounds)]

use poem_openapi::{payload::PlainText, OpenApi, Tags};

pub mod routes;
pub mod auth;
