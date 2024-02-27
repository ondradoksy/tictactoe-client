use serde::Deserialize;

use crate::utils::from_json;

#[derive(Deserialize)]
pub(crate) struct PlayerImageResponse {
    pub id: u32,
    pub image: String,
}

impl PlayerImageResponse {
    pub fn from_json(text: &str) -> Result<Self, String> {
        from_json(text)
    }
}
