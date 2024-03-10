use std::{ ops::Add, rc::Rc, cell::RefCell };

use js_sys::JSON;
use wasm_bindgen::{ closure::Closure, JsCast };
use web_sys::{ WebSocket, MessageEvent, ErrorEvent, HtmlElement };

use crate::{
    debug,
    error,
    game::Game,
    gameinfo::GameInfo,
    gamejoindata::GameJoinData,
    gamemessageevent::GameMessageEvent,
    grid::Grid,
    log,
    player::{ merge_players, set_image, Player },
    playerimageresponse::PlayerImageResponse,
    playermove::PlayerMove,
    utils::{ add_event_listener, document, games_div, get_element_by_id, players_div, set_timeout },
    warn,
};

pub(crate) fn start_websocket(
    current_game: &Rc<RefCell<Option<GameInfo>>>,
    game: &Rc<RefCell<Option<Game>>>,
    players: &Rc<RefCell<Vec<Player>>>
) -> WebSocket {
    let ip = web_sys::window().unwrap().location().hostname().unwrap();
    let ws = WebSocket::new(format!("ws://{}:9001/", ip).as_str()).unwrap();

    log!("Connecting to {}", format!("wss://{}:9001/", ip));

    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let current_game_clone = current_game.clone();

    let mut game_list: Rc<RefCell<Vec<GameInfo>>> = Rc::new(RefCell::new(Vec::new()));
    let game_clone = game.clone();
    let players_clone = players.clone();

    let ws_clone = ws.clone();
    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        // Handle difference Text/Binary,...
        if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            log!("message event, received arraybuffer: {:?}", abuf);
        } else if let Ok(blob) = e.data().dyn_into::<web_sys::Blob>() {
            log!("message event, received blob: {:?}", blob);
        } else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            let event_result = GameMessageEvent::from_json(txt.as_string().unwrap().as_str());

            if event_result.is_ok() {
                let event = event_result.unwrap();
                match event.event.as_str() {
                    "players" => {
                        update_player_list(
                            &mut players_clone.borrow_mut(),
                            event.content.as_str(),
                            &current_game_clone.borrow()
                        );
                    }
                    "games" => {
                        update_game_list(event.content.as_str(), &ws_clone, &mut game_list);
                    }
                    "joined_game" => {
                        joined_game(
                            event.content.as_str(),
                            &mut current_game_clone.borrow_mut(),
                            &game_list,
                            &players_clone.borrow()
                        );
                    }
                    "new_move" => { new_move(event.content.as_str(), &mut game_clone.borrow_mut()) }
                    "current_state" => {
                        start_game(
                            event.content.as_str(),
                            &mut game_clone.borrow_mut(),
                            &ws_clone,
                            &players_clone
                        );
                    }
                    "player_image" => {
                        update_player_image(
                            &mut players_clone.borrow_mut(),
                            event.content.as_str()
                        );
                    }
                    _ => {
                        warn!("Unrecognized event: {:?} {:?}", event.event, event.content);
                    }
                }
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

fn update_player_image(players: &mut Vec<Player>, content: &str) {
    let result = PlayerImageResponse::from_json(content);
    if result.is_err() {
        error!("{:?}", result.err());
        return;
    }

    set_image(players, result.unwrap());
}

fn update_player_list(players: &mut Vec<Player>, content: &str, current_game: &Option<GameInfo>) {
    let player_list: Vec<Player> = serde_wasm_bindgen
        ::from_value(JSON::parse(content).unwrap())
        .unwrap();

    merge_players(players, &player_list);

    display_players(&player_list, current_game);
}

fn display_players(player_list: &Vec<Player>, current_game: &Option<GameInfo>) {
    let list: HtmlElement = players_div();
    list.set_inner_html("");

    let list_game = get_element_by_id("game-player-list");
    list_game.set_inner_html("");

    for p in player_list {
        debug!("Player: {} {} {:?} {}", p.id, p.name, p.joined_game_id, p.ready);
        let div = document().create_element("div").expect("Unable to create div");
        div.set_text_content(Some(format!("{}#{}", p.name, p.id).as_str()));
        list.append_child(&div).expect("Unable to add player to list");
        if
            current_game.is_none() ||
            p.joined_game_id.is_none() ||
            p.joined_game_id.unwrap() != current_game.as_ref().unwrap().id
        {
            continue;
        }
        list_game.append_child(&div).expect("Unable to add player to list");
    }
}
fn update_game_list(content: &str, ws: &WebSocket, game_list: &Rc<RefCell<Vec<GameInfo>>>) {
    let games: Vec<GameInfo> = serde_wasm_bindgen
        ::from_value(JSON::parse(content).unwrap())
        .unwrap();
    *game_list.borrow_mut() = games;

    let list = games_div();
    list.set_inner_html("");

    let game_list_clone = game_list.clone();

    for i in 0..game_list.borrow().len() {
        let game_list_borrow = game_list_clone.borrow();
        let g = game_list_borrow[i].clone();
        debug!(
            "GameInfo: {} {} {} {} {} {} {} {}",
            g.id,
            g.player_list.len(),
            g.creator,
            g.current_turn,
            g.hotjoin,
            g.player_limit,
            g.running,
            g.length_to_win
        );
        let div = document().create_element("div").expect("Unable to create div");
        div.set_text_content(Some(format!("{} - {} players", g.id, g.player_list.len()).as_str()));

        let ws_clone = ws.clone();
        add_event_listener(&div, "click", move |_| {
            send(&ws_clone, "join_game", GameJoinData::new(g.id).to_string().as_str());
        });

        list.append_child(&div).expect("Unable to add game to list");
    }
}

fn joined_game(
    content: &str,
    current_game: &mut Option<GameInfo>,
    game_list: &Rc<RefCell<Vec<GameInfo>>>,
    player_list: &Vec<Player>
) {
    let data_result = GameJoinData::from_json(&content);
    if data_result.is_err() {
        error!("Unable to parse game join data info, either client or server may be out of date.");
        return;
    }
    let data = data_result.unwrap();
    log!("Joined game: {}", data.id);

    let menu = get_element_by_id("menu");
    let lobby = get_element_by_id("lobby");

    menu.set_class_name(menu.class_name().add(" hidden").as_str());
    lobby.set_class_name("fullscreen");

    let index = game_list
        .borrow()
        .iter()
        .position(|p| { p.id == data.id })
        .unwrap();

    *current_game = Some(game_list.borrow()[index].clone());

    // Display game info
    get_element_by_id("game-id").set_text_content(
        Some(current_game.as_ref().unwrap().id.to_string().as_str())
    );
    get_element_by_id("game-size-w").set_text_content(
        Some(current_game.as_ref().unwrap().width.to_string().as_str())
    );
    get_element_by_id("game-size-h").set_text_content(
        Some(current_game.as_ref().unwrap().height.to_string().as_str())
    );
    get_element_by_id("game-hotjoin").set_text_content(
        Some(current_game.as_ref().unwrap().hotjoin.to_string().as_str())
    );
    get_element_by_id("game-win-length").set_text_content(
        Some(current_game.as_ref().unwrap().length_to_win.to_string().as_str())
    );

    display_players(player_list, current_game);
}

fn start_game(
    content: &str,
    game: &mut Option<Game>,
    ws: &WebSocket,
    players: &Rc<RefCell<Vec<Player>>>
) {
    let grid_result = Grid::from_json(content);
    if grid_result.is_err() {
        error!("{}", grid_result.err().unwrap());
        return;
    }
    let grid = grid_result.unwrap();
    log!("{:?}", grid);

    *game = Some(Game::new("game", grid, ws, players));
    let lobby = get_element_by_id("lobby");
    lobby.set_class_name(lobby.class_name().add(" hidden").as_str());

    let game_container = get_element_by_id("game-container");
    game_container.set_class_name("");
}

fn new_move(content: &str, game: &mut Option<Game>) {
    if game.is_none() {
        return;
    }

    let m = PlayerMove::from_json(content).expect("Unable to parse JSON");
    game.as_mut().unwrap().add_move(m);
}

pub fn send(ws: &WebSocket, event: &str, content: &str) {
    let msg = GameMessageEvent::new(event, content);

    // Check if websocket is open
    if ws.ready_state() != 1 {
        // Wait for websocket to be open
        let ws_clone = ws.clone();
        let cb = Closure::wrap(
            Box::new(move || {
                send(&ws_clone, msg.event.as_str(), msg.content.as_str());
            }) as Box<dyn FnMut()>
        );
        set_timeout(&cb, 1000);
        cb.forget();
        return;
    }
    let text = msg.to_string();
    ws.send_with_str(text.as_str()).expect("Unable to send message");
}
