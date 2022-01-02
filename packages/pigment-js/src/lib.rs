extern crate canvas_lms as canvas;

mod utils;

use ebauche_rpc::message::DResource;
use pigment::DSelector;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

fn foo() -> JsValue {
    JsValue::from_serde(&DSelector::All(Default::default())).unwrap()
}
