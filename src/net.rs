use js_sys::JSON;
use serde::{ Deserialize, Serialize };
use wasm_bindgen::{ closure::Closure, JsCast, JsValue };
use web_sys::{ WebSocket, MessageEvent, ErrorEvent };

use crate::log;

pub fn start_websocket() -> WebSocket {
    let ip = web_sys::window().unwrap().location().hostname().unwrap();
    let ws = WebSocket::new(format!("ws://{}:9001/", ip).as_str()).unwrap();

    log!("Connecting to {}", format!("wss://{}:9001/", ip));

    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        // Handle difference Text/Binary,...
        if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            log!("message event, received arraybuffer: {:?}", abuf);
        } else if let Ok(blob) = e.data().dyn_into::<web_sys::Blob>() {
            log!("message event, received blob: {:?}", blob);
        } else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            log!("message event, received Text: {:?}", txt);
            let event_result = GameMessageEvent::from_json(txt.as_string().unwrap().as_str());
            if event_result.is_ok() {
                let event = event_result.unwrap();
                log!("{:?} {:?}", event.event, event.content);
            }
        } else {
            log!("message event, received Unknown: {:?}", e.data());
        }
    });
    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
        log!("error event: {:?}", e);
    });
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let cloned_ws = ws.clone();
    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        log!("socket opened");
        match cloned_ws.send_with_str("{\"event\":\"players\",\"content\":\"\"}") {
            Ok(_) => log!("message successfully sent"),
            Err(err) => log!("error sending message: {:?}", err),
        }
    });
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();
    ws
}

pub fn from_jsvalue<T>(value: JsValue) -> Result<T, String> where T: serde::de::DeserializeOwned {
    let result: Result<T, serde_wasm_bindgen::Error> = serde_wasm_bindgen::from_value(value);
    if result.is_ok() {
        return Ok(result.unwrap());
    }
    let err_string = result.err().unwrap().to_string();
    Err(err_string)
}

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
    pub fn new_empty() -> Self {
        Self {
            event: String::from(""),
            content: String::from(""),
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
    pub fn is_empty(&self) -> bool {
        self.event.as_str() == ""
    }
}