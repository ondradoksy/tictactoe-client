use serde::{ Serialize, Deserialize };
use wasm_bindgen::{ JsCast, closure::Closure };
use web_sys::{ HtmlElement, Window, Document };

extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!( $( $t )* ).into())
    };
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn now() -> f64 {
    web_sys
        ::window()
        .expect("should have a Window")
        .performance()
        .expect("should have a Performance")
        .now()
}

pub fn generate_random_u32(min: u32, max: u32) -> u32 {
    let mut rand_array: [u8; 4] = [0u8; 4];
    let crypto = web_sys::window().unwrap().crypto().unwrap();

    crypto.get_random_values_with_u8_array(&mut rand_array).unwrap();

    // Convert the random bytes to an i32 value.
    let random_usize = u32::from_be_bytes(rand_array);

    // Return a random i32 value between the specified min and max values.
    (random_usize % (max - min + 1)) + min
}

pub fn window() -> Window {
    web_sys::window().expect("no global 'window' exists")
}
pub fn document() -> Document {
    window().document().expect("should have a document on window")
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Size {
    pub x: i32,
    pub y: i32,
}
impl Size {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x: x,
            y: y,
        }
    }
}

pub fn players_div() -> HtmlElement {
    document()
        .get_element_by_id("player-list")
        .expect("Player list not found")
        .dyn_into()
        .expect("Not HtmlElement type")
}

pub fn games_div() -> HtmlElement {
    document()
        .get_element_by_id("game-list")
        .expect("Game list not found")
        .dyn_into()
        .expect("Not HtmlElement type")
}

pub fn set_interval(f: &Closure<dyn FnMut()>, interval_ms: i32) -> i32 {
    window()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            f.as_ref().unchecked_ref(),
            interval_ms
        )
        .expect("should register `setInterval` OK")
}
