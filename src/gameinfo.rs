use serde::Deserialize;

use crate::player::Player;

#[derive(Deserialize)]
pub(crate) struct GameInfo {
    pub id: u32,
    pub player_list: Vec<Player>,
    pub creator: Player,
    pub current_turn: u32,
    pub hotjoin: bool,
    pub player_limit: u32,
    pub running: bool,
    pub length_to_win: u32,
}
