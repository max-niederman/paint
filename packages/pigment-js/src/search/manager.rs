use wasm_bindgen::prelude::*;
use super::Query;

#[wasm_bindgen]
pub struct Manager {
    query: Query,
}
