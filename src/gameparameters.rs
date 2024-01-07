use js_sys::JSON;
use serde::Serialize;

use crate::utils::Size;

#[derive(Serialize)]
pub(crate) struct GameParameters {
    pub size: Size,
    pub hotjoin: bool,
    pub player_limit: u32,
    pub length_to_win: u32,
}
impl GameParameters {
    pub fn new(size: Size, hotjoin: bool, player_limit: u32, length_to_win: u32) -> Self {
        Self {
            size: size,
            hotjoin: hotjoin,
            player_limit: player_limit,
            length_to_win: length_to_win,
        }
    }
    pub fn to_json(&self) -> String {
        JSON::stringify(&serde_wasm_bindgen::to_value(&self).expect("Unable to serialize"))
            .expect("Unable to stringify")
            .as_string()
            .expect("Not string")
    }
}
