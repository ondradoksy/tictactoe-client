use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Player {
    pub id: u32,
    pub joined_game_id: Option<u32>,
    pub ready: bool,
    pub name: String,
}
impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
