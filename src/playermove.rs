use serde::Deserialize;

use crate::utils::{ Size, from_json };

#[derive(Deserialize, Debug)]
pub struct PlayerMove {
    pub player: u32,
    pub position: Size,
}
impl PlayerMove {
    pub fn from_json(text: &str) -> Result<Self, String> {
        from_json(text)
    }
}
