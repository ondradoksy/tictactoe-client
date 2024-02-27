use serde::Deserialize;

use crate::{ utils::{ Size, from_json }, playermove::PlayerMove };

#[derive(Deserialize, Debug)]
pub struct Grid {
    pub size: Size,
    pub moves: Vec<PlayerMove>,
}
impl Grid {
    pub fn from_json(text: &str) -> Result<Self, String> {
        from_json(text)
    }
    /// Returns None if the tile is empty, otherwise returns the player's id.
    pub fn get_pos(&self, pos: &Size) -> Option<u32> {
        let index = self.get_index(pos);

        if index.is_some() {
            return Some(self.moves[index.unwrap()].player);
        }
        None
    }
    pub fn add(&mut self, m: PlayerMove) {
        self.moves.push(m);
    }
    fn get_index(&self, pos: &Size) -> Option<usize> {
        let index = self.moves
            .iter()
            .rev()
            .position(|m| m.position == *pos);
        if index.is_none() {
            return None;
        }
        Some(self.moves.len() - index.unwrap() - 1)
    }
    pub fn is_empty(&self, pos: &Size) -> bool {
        self.get_pos(pos).is_none()
    }
    pub fn is_valid_move(&self, pos: &Size) -> bool {
        self.is_empty(pos) && pos.x < self.size.x && pos.y < self.size.y
    }
}
