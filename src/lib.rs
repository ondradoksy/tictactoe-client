mod utils;
pub mod game;
mod net;

use std::{ cell::RefCell, rc::Rc, convert::TryInto };

use net::start_websocket;
use utils::{ set_panic_hook, window };
use wasm_bindgen::prelude::*;
use web_sys::MouseEvent;

use crate::{ game::Game, utils::document };

extern crate js_sys;
extern crate web_sys;

#[wasm_bindgen(start)]
pub fn init() {
    log!("Starting...");
    set_panic_hook();
    start_websocket();

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let canvas_id = "game";

    let canvas = document().get_element_by_id(canvas_id).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let game = Rc::new(RefCell::new(Game::new(canvas_id, Some(10), Some(10))));

    let game_clone = game.clone();
    let cb = Closure::wrap(
        Box::new(move |e: MouseEvent| {
            game_clone.borrow_mut().on_mouse_move(e);
        }) as Box<dyn FnMut(_)>
    );
    canvas
        .add_event_listener_with_callback("mousemove", &cb.as_ref().unchecked_ref())
        .expect("Something went wrong");
    cb.forget();

    let game_clone = game.clone();
    let cb = Closure::wrap(
        Box::new(move |_e: MouseEvent| {
            game_clone.borrow_mut().click();
        }) as Box<dyn FnMut(_)>
    );
    canvas
        .add_event_listener_with_callback("click", &cb.as_ref().unchecked_ref())
        .expect("Something went wrong");
    cb.forget();

    *g.borrow_mut() = Some(
        Closure::new(move || {
            if
                canvas.width() != canvas.client_width().try_into().unwrap() ||
                canvas.height() != canvas.client_height().try_into().unwrap()
            {
                canvas.set_width(canvas.client_width().try_into().unwrap());
                canvas.set_height(canvas.client_height().try_into().unwrap());
            }
            game.borrow_mut().render();

            request_animation_frame(f.borrow().as_ref().unwrap());
        })
    );

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
