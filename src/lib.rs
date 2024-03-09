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
mod gameobject;
mod texture;

use std::{ cell::RefCell, convert::TryInto, rc::Rc };

use gameparameters::GameParameters;
use net::{ start_websocket, send };
use utils::{ get_element_by_id, get_elements_by_class_name, set_panic_hook, window, Size };
use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlCanvasElement,
    HtmlElement,
    HtmlSelectElement,
    MouseEvent,
    WebSocket,
    WheelEvent,
};

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
    register_tabs();
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

    let ws_clone = ws.clone();
    let cb = Closure::wrap(
        Box::new(move || {
            let select = get_element_by_id("game-bot-type")
                .dyn_into::<HtmlSelectElement>()
                .expect("Not a select element");

            send(
                &ws_clone,
                "add_bot",
                select
                    .item(select.selected_index().try_into().unwrap())
                    .expect("No element selected")
                    .get_attribute("value")
                    .expect("No value")
                    .as_str()
            );
        }) as Box<dyn FnMut()>
    );
    get_element_by_id("game-bot-btn")
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

fn register_tabs() {
    let elements = get_elements_by_class_name("tab");
    for i in 0..elements.length() {
        let tab = elements.item(i).unwrap();
        let tab_name: HtmlElement = tab
            .get_elements_by_class_name("tab-name")
            .item(0)
            .expect("Missing tab-name")
            .dyn_into()
            .expect("Not HtmlElement type");
        let tab_content_container: HtmlElement = tab
            .get_elements_by_class_name("tab-content-container")
            .item(0)
            .expect("Missing tab-content-container")
            .dyn_into()
            .expect("Not HtmlElement type");
        let tab_content: HtmlElement = tab
            .get_elements_by_class_name("tab-content")
            .item(0)
            .expect("Missing tab-content")
            .dyn_into()
            .expect("Not HtmlElement type");

        tab_content_container
            .style()
            .set_property("min-width", format!("{}px", tab_name.offset_width()).as_str())
            .expect("Could not set min-width");

        let tab_name_clone = tab_name.clone();

        let cb = Closure::wrap(
            Box::new(move || {
                if tab_content_container.style().get_property_value("height").unwrap() == "" {
                    let new_width = if
                        tab_content.offset_width() > tab_name_clone.offset_width() + 50
                    {
                        tab_content.offset_width()
                    } else {
                        tab_name_clone.offset_width() + 50
                    };

                    tab_content_container
                        .style()
                        .set_property("width", format!("{}px", new_width).as_str())
                        .expect("Could not set width");
                    tab_content_container
                        .style()
                        .set_property(
                            "min-width",
                            format!("{}px", tab_name_clone.offset_width()).as_str()
                        )
                        .expect("Could not set min-width");
                    tab_content_container
                        .style()
                        .set_property("border-radius", "1em 0 0 0")
                        .expect("Could not set border-radius");
                    tab_content_container
                        .style()
                        .set_property(
                            "height",
                            format!("{}px", tab_content.offset_height()).as_str()
                        )
                        .expect("Could not set height");
                } else {
                    tab_content_container
                        .style()
                        .set_property(
                            "width",
                            format!("{}px", tab_name_clone.offset_width()).as_str()
                        )
                        .expect("Could not set width");
                    tab_content_container
                        .style()
                        .remove_property("border-radius")
                        .expect("Could not remove border-radius");
                    tab_content_container
                        .style()
                        .remove_property("height")
                        .expect("Could not remove height");
                }
            }) as Box<dyn FnMut()>
        );

        tab_name
            .add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
            .expect("Unable to register event");

        cb.forget();
    }
}
