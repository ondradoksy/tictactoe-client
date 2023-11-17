mod utils;
pub mod game;

use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

extern crate js_sys;
extern crate web_sys;

#[wasm_bindgen(start)]
pub fn init() {
    set_panic_hook();
}
