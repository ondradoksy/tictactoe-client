use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub(crate) struct GameInfo {
    pub id: u32,
    pub player_list: Vec<u32>,
    pub creator: u32,
    pub current_turn: u32,
    pub hotjoin: bool,
    pub player_limit: u32,
    pub running: bool,
    pub win_length: u32,
    pub width: u32,
    pub height: u32,
}
