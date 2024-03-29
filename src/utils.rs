use js_sys::JSON;
use serde::{ Serialize, Deserialize };
use wasm_bindgen::{ JsCast, closure::Closure, JsValue };
use web_sys::{ Document, Element, Event, HtmlCollection, HtmlElement, HtmlInputElement, Window };

extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!( $( $t )* ).into())
    };
}
#[macro_export]
macro_rules! warn {
    ($($t:tt)*) => {
        web_sys::console::warn_1(&format!( $( $t )* ).into())
    };
}
#[macro_export]
macro_rules! error {
    ($($t:tt)*) => {
        web_sys::console::error_1(&format!( $( $t )* ).into())
    };
}
#[macro_export]
macro_rules! debug {
    ($($t:tt)*) => {
        web_sys::console::debug_1(&format!( $( $t )* ).into())
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

pub fn window() -> Window {
    web_sys::window().expect("no global 'window' exists")
}
pub fn document() -> Document {
    window().document().expect("should have a document on window")
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
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
    pub fn to_json(&self) -> String {
        (*self).into()
    }
}
impl From<Size> for String {
    fn from(value: Size) -> Self {
        serde_wasm_bindgen
            ::from_value(
                JSON::stringify(
                    &serde_wasm_bindgen::to_value(&value).expect("Could not convert to JsValue")
                )
                    .expect("Could not stringify")
                    .into()
            )
            .expect("Could not convert from JsValue")
    }
}

pub fn players_div() -> HtmlElement {
    get_element_by_id("player-list")
}

pub fn games_div() -> HtmlElement {
    get_element_by_id("game-list")
}

pub fn set_timeout(f: &Closure<dyn FnMut()>, interval_ms: i32) -> i32 {
    window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            f.as_ref().unchecked_ref(),
            interval_ms
        )
        .expect("should register `setTimeout` OK")
}

pub fn get_element_by_id(id: &str) -> HtmlElement {
    document()
        .get_element_by_id(id)
        .expect("Element not found")
        .dyn_into()
        .expect("Not HtmlElement type")
}
pub fn get_input_element_by_id(id: &str) -> HtmlInputElement {
    get_element_by_id(id).dyn_into().expect("Not HtmlInputElement type")
}

pub fn get_elements_by_class_name(name: &str) -> HtmlCollection {
    document().get_elements_by_class_name(name)
}

pub fn add_event_listener(element: &Element, event: &str, f: impl Fn(Event) + 'static) {
    let cb = Closure::wrap(Box::new(f) as Box<dyn FnMut(_)>);
    element
        .add_event_listener_with_callback(event, &cb.as_ref().unchecked_ref())
        .expect("Something went wrong");
    cb.forget();
}

pub fn from_jsvalue<T>(value: JsValue) -> Result<T, String> where T: serde::de::DeserializeOwned {
    let result: Result<T, serde_wasm_bindgen::Error> = serde_wasm_bindgen::from_value(value);
    if result.is_ok() {
        return Ok(result.unwrap());
    }
    let err_string = result.err().unwrap().to_string();
    Err(err_string)
}

pub fn from_json<T>(text: &str) -> Result<T, String> where T: serde::de::DeserializeOwned {
    let result: Result<T, serde_wasm_bindgen::Error> = serde_wasm_bindgen::from_value(
        JSON::parse(text).expect("Unable to parse")
    );
    if result.is_ok() {
        return Ok(result.unwrap());
    }
    let err_string = result.err().unwrap().to_string();
    Err(err_string)
}
