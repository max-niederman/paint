#![feature(bound_map)]
#![feature(btree_drain_filter)]
#![feature(generic_associated_types)]

extern crate canvas_lms as canvas;

pub mod search;
pub mod store;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn initialize() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    #[cfg(feature = "tracing-wasm")]
    tracing_wasm::set_as_global_default();
}

// TODO: test alternative allocators
