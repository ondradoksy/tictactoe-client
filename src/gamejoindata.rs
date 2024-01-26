use js_sys::JSON;
use serde::{ Deserialize, Serialize };

use crate::utils::from_json;

#[derive(Deserialize, Serialize)]
pub(crate) struct GameJoinData {
    pub id: u32,
}
impl GameJoinData {
    pub fn new(id: u32) -> Self {
        Self {
            id: id,
        }
    }
    pub fn from_json(text: &str) -> Result<Self, String> {
        from_json(text)
    }
}
impl From<GameJoinData> for String {
    fn from(value: GameJoinData) -> Self {
        JSON::stringify(&serde_wasm_bindgen::to_value(&value).expect("Unable to serialize"))
            .expect("Unable to stringify")
            .as_string()
            .expect("Not string")
    }
}

#[test]
fn game_join_data_test() {
    assert!(true);
}
