use std::{ cell::RefCell, rc::Rc };

use serde::Deserialize;

use crate::playerimageresponse::PlayerImageResponse;

#[derive(Deserialize, Clone)]
pub(crate) struct Player {
    pub id: u32,
    pub joined_game_id: Option<u32>,
    pub ready: bool,
    pub name: String,
    #[serde(skip_serializing)]
    image: Option<String>,
}
impl Player {
    pub fn get_image(&self) -> Option<String> {
        if self.image.is_none() {
        }
        self.image.clone()
    }
    pub fn set_image(&mut self, image: String) {
        self.image = Some(image);
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
