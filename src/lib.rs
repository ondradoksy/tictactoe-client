mod utils;
pub mod game;
mod net;
mod mouse;
mod player;
mod gameinfo;
mod gameparameters;
mod gamejoindata;
mod gamemessageevent;
mod grid;
mod playermove;
mod playerimageresponse;

use std::{ cell::RefCell, rc::Rc, convert::TryInto };

use gameparameters::GameParameters;
use net::{ start_websocket, send };
use utils::{ set_panic_hook, window, get_element_by_id, Size };
use wasm_bindgen::prelude::*;
use web_sys::{ MouseEvent, HtmlCanvasElement, WheelEvent, WebSocket };

use crate::{ game::Game, gameinfo::GameInfo, player::Player, utils::document };

extern crate js_sys;
extern crate web_sys;

#[wasm_bindgen(start)]
pub fn init() {
    log!("Starting...");
    set_panic_hook();

    let canvas_id = "game";

    let canvas = document().get_element_by_id(canvas_id).unwrap();
    let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

    let current_game: Rc<RefCell<Option<GameInfo>>> = Rc::new(RefCell::new(None));

    let game: Rc<RefCell<Option<Game>>> = Rc::new(RefCell::new(None));

    let players: Rc<RefCell<Vec<Player>>> = Rc::new(RefCell::new(Vec::new()));

    let ws = start_websocket(&current_game, &game, &players);
    //let _ = ws.send_with_str("{\"event\":\"players\",\"content\":\"\"}");
    update_menu(&ws); // Initial menu update

    register_inputs(&game, &canvas, &ws);

    start_game_render(&game, &canvas);

    //Menu should be automatically updated by server on change
    //start_menu_update_timer(&ws);

    register_menu_buttons(&ws);
    register_lobby_buttons(&ws);
}

fn register_lobby_buttons(ws: &WebSocket) {
    let ws_clone = ws.clone();
    let cb = Closure::wrap(
        Box::new(move || {
            send(&ws_clone, "ready", "");
        }) as Box<dyn FnMut()>
    );
    get_element_by_id("ready-btn")
        .add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
        .expect("Unable to register event");
    cb.forget();
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

fn update_menu(ws: &WebSocket) {
    log!("Fetching players");
    send(&ws, "players", "");
    log!("Fetching games");
    send(&ws, "games", "");
}

fn start_game_render(game: &Rc<RefCell<Option<Game>>>, canvas: &HtmlCanvasElement) {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let game_clone = game.clone();
    let canvas_clone = canvas.clone();

    *g.borrow_mut() = Some(
        Closure::new(move || {
            if
                canvas_clone.width() != canvas_clone.client_width().try_into().unwrap() ||
                canvas_clone.height() != canvas_clone.client_height().try_into().unwrap()
            {
                canvas_clone.set_width(canvas_clone.client_width().try_into().unwrap());
                canvas_clone.set_height(canvas_clone.client_height().try_into().unwrap());
            }
            let mut game_borrowed = game_clone.borrow_mut();
            if game_borrowed.is_some() {
                game_borrowed.as_mut().unwrap().render();
            }

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

fn register_inputs(game: &Rc<RefCell<Option<Game>>>, canvas: &HtmlCanvasElement, ws: &WebSocket) {
    let ws_clone = ws.clone();

    // Mouse move
    let game_clone = game.clone();
    let cb = Closure::wrap(
        Box::new(move |e: MouseEvent| {
            let mut game_borrowed = game_clone.borrow_mut();
            if game_borrowed.is_none() {
                return;
            }
            game_borrowed.as_mut().unwrap().on_mouse_move(e);
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
            let mut game_borrowed = game_clone.borrow_mut();
            if game_borrowed.is_none() {
                return;
            }
            game_borrowed.as_mut().unwrap().on_mouse_down(e);
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
            let mut game_borrowed = game_clone.borrow_mut();
            if game_borrowed.is_none() {
                return;
            }
            game_borrowed.as_mut().unwrap().on_mouse_up(e, &ws_clone);
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
            let mut game_borrowed = game_clone.borrow_mut();
            if game_borrowed.is_none() {
                return;
            }
            game_borrowed.as_mut().unwrap().on_scroll(e);
        }) as Box<dyn FnMut(_)>
    );
    canvas
        .add_event_listener_with_callback("wheel", &cb.as_ref().unchecked_ref())
        .expect("Something went wrong");
    cb.forget();
}
