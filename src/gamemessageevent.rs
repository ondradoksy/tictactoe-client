use js_sys::JSON;
use serde::{ Serialize, Deserialize };
use wasm_bindgen::JsValue;

use crate::utils::from_jsvalue;

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct GameMessageEvent {
    pub event: String,
    pub content: String,
}
impl GameMessageEvent {
    pub fn new(event: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            content: content.into(),
        }
    }
    pub fn from_json(text: &str) -> Result<Self, String> {
        let result = JSON::parse(text);
        if result.is_err() {
            return Err(result.err().unwrap().as_string().unwrap());
        }
        Self::from_jsvalue(result.unwrap())
    }
    pub fn from_jsvalue(value: JsValue) -> Result<Self, String> {
        from_jsvalue(value)
    }
    pub fn to_string(&self) -> String {
        (*self).clone().into()
    }
}

impl From<GameMessageEvent> for String {
    fn from(value: GameMessageEvent) -> Self {
        JSON::stringify(&serde_wasm_bindgen::to_value(&value).expect("Unable to serialize"))
            .expect("Unable to stringify")
            .as_string()
            .expect("Not string")
    }
}
