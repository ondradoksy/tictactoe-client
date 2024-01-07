mod utils;
pub mod game;
mod net;
mod mouse;
mod player;
mod gameinfo;
mod gameparameters;

use std::{ cell::RefCell, rc::Rc, convert::TryInto };

use gameparameters::GameParameters;
use net::{ start_websocket, send };
use utils::{ set_panic_hook, window, set_interval, get_element_by_id, Size };
use wasm_bindgen::prelude::*;
use web_sys::{ MouseEvent, HtmlCanvasElement, WheelEvent, WebSocket };

use crate::{ game::Game, utils::document };

extern crate js_sys;
extern crate web_sys;

#[wasm_bindgen(start)]
pub fn init() {
    log!("Starting...");
    set_panic_hook();

    let canvas_id = "game";

    let canvas = document().get_element_by_id(canvas_id).unwrap();
    let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

    let game = Rc::new(RefCell::new(Game::new(canvas_id, Some(10), Some(10))));

    register_inputs(&game, &canvas);

    start_game_render(game, canvas);

    let ws = start_websocket();
    let _ = ws.send_with_str("{\"event\":\"players\",\"content\":\"\"}");
    start_menu_update_timer(&ws);
    register_menu_buttons(&ws);
}

fn register_menu_buttons(ws: &WebSocket) {
    let ws_clone = ws.clone();
    let cb = Closure::wrap(
        Box::new(move || {
            send(
                &ws_clone,
                "create_game",
                GameParameters::new(Size::new(10, 10), true, 10, 5).to_json().as_str()
            );
        }) as Box<dyn FnMut()>
    );
    get_element_by_id("new-game-btn")
        .add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
        .expect("Unable to register event");
    cb.forget();
}

fn start_menu_update_timer(ws: &WebSocket) {
    let ws_clone = ws.clone();
    let cb = Closure::wrap(
        Box::new(move || {
            send(&ws_clone, "players", "");
            send(&ws_clone, "games", "");
        }) as Box<dyn FnMut()>
    );
    set_interval(&cb, 1000);
    cb.forget();
}

fn start_game_render(game: Rc<RefCell<Game>>, canvas: HtmlCanvasElement) {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

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

fn register_inputs(game: &Rc<RefCell<Game>>, canvas: &HtmlCanvasElement) {
    // Mouse move
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

    // Mouse down
    let game_clone = game.clone();
    let cb = Closure::wrap(
        Box::new(move |e: MouseEvent| {
            game_clone.borrow_mut().on_mouse_down(e);
        }) as Box<dyn FnMut(_)>
    );
    canvas
        .add_event_listener_with_callback("mousedown", &cb.as_ref().unchecked_ref())
        .expect("Something went wrong");
    cb.forget();

    // Mouse up
    let game_clone = game.clone();
    let cb = Closure::wrap(
        Box::new(move |e: MouseEvent| {
            game_clone.borrow_mut().on_mouse_up(e);
        }) as Box<dyn FnMut(_)>
    );
    canvas
        .add_event_listener_with_callback("mouseup", &cb.as_ref().unchecked_ref())
        .expect("Something went wrong");
    cb.forget();

    // Scroll
    let game_clone = game.clone();
    let cb = Closure::wrap(
        Box::new(move |e: WheelEvent| {
            game_clone.borrow_mut().on_scroll(e);
        }) as Box<dyn FnMut(_)>
    );
    canvas
        .add_event_listener_with_callback("wheel", &cb.as_ref().unchecked_ref())
        .expect("Something went wrong");
    cb.forget();
}
