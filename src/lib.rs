mod utils;
pub mod game;
mod net;

use net::start_websocket;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

extern crate js_sys;
extern crate web_sys;

#[wasm_bindgen(start)]
pub fn init() {
    log!("Starting...");
    set_panic_hook();
    start_websocket();
}
