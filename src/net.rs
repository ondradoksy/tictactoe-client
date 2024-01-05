use wasm_bindgen::{ closure::Closure, JsCast };
use web_sys::{ WebSocket, MessageEvent, ErrorEvent };

use crate::log;

pub fn start_websocket() {
    let ip = web_sys::window().unwrap().location().hostname().unwrap();
    let ws = WebSocket::new(format!("ws://{}:9001/", ip).as_str()).unwrap();

    log!("Connecting to {}", format!("wss://{}:9001/", ip));

    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
    let cloned_ws = ws.clone();
    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        // Handle difference Text/Binary,...
        if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            log!("message event, received arraybuffer: {:?}", abuf);
        } else if let Ok(blob) = e.data().dyn_into::<web_sys::Blob>() {
            log!("message event, received blob: {:?}", blob);
        } else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            log!("message event, received Text: {:?}", txt);
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
}
