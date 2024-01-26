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
    pub length_to_win: u32,
}
