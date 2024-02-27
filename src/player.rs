use serde::Deserialize;
use web_sys::{ HtmlImageElement, WebSocket };

use crate::{ net::send, playerimageresponse::PlayerImageResponse };

#[derive(Deserialize, Clone)]
pub(crate) struct Player {
    pub id: u32,
    pub joined_game_id: Option<u32>,
    pub ready: bool,
    pub name: String,
    #[serde(skip_deserializing)]
    image: Option<HtmlImageElement>,
}
impl Player {
    pub fn get_image(&mut self, ws: &WebSocket) -> HtmlImageElement {
        if self.image.is_none() {
            send(ws, "get_image", self.id.to_string().as_str());
            self.image = Some(HtmlImageElement::new().unwrap());
        }
        self.image.clone().unwrap()
    }
    pub fn set_image(&mut self, image: String) {
        self.image = Some(HtmlImageElement::new().unwrap());
        self.image.clone().unwrap().set_src(format!("data:image/png;base64,{}", image).as_str());
    }
}
impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub(crate) fn merge_players(players: &mut Vec<Player>, new_players: &Vec<Player>) {
    for p in new_players {
        if players.contains(p) {
        } else {
            players.push(p.clone());
        }
    }
}

pub(crate) fn set_image(players: &mut Vec<Player>, response: PlayerImageResponse) {
    let result = players.iter().position(|p| p.id == response.id);
    if result.is_none() {
        return;
    }
    players[result.unwrap()].set_image(response.image);
}
