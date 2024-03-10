use std::convert::TryInto;

use serde::Deserialize;

use crate::{ utils::{ Size, from_json }, playermove::PlayerMove };

#[derive(Deserialize, Debug)]
pub struct Grid {
    pub size: Size,
    pub moves: Vec<PlayerMove>,
    #[serde(skip_deserializing)]
    cache: Vec<Option<i32>>,
}
impl Grid {
    pub fn from_json(text: &str) -> Result<Self, String> {
        from_json(text)
    }
    /// Returns None if the tile is empty, otherwise returns the player's id.
    pub fn get_pos(&mut self, pos: &Size) -> Option<i32> {
        self.check_cache_integrity();
        let cache_result =
            self.cache
                [
                    <i32 as TryInto<usize>>
                        ::try_into(pos.y * self.size.x + pos.x)
                        .expect("Could not convert to usize")
                ];
        if cache_result.is_some() {
            return cache_result;
        }

        let index = self.get_index(pos);

        if index.is_some() {
            let value = self.moves[index.unwrap()].player;
            self.update_cache(pos.x, pos.y, value);
            return Some(value);
        }
        None
    }
    pub fn add(&mut self, m: PlayerMove) {
        self.check_cache_integrity();
        self.update_cache(m.position.x, m.position.y, m.player);
        self.moves.push(m);
    }
    fn get_index(&mut self, pos: &Size) -> Option<usize> {
        let index = self.moves
            .iter()
            .rev()
            .position(|m| m.position == *pos);
        if index.is_none() {
            return None;
        }
        Some(self.moves.len() - index.unwrap() - 1)
    }
    pub fn is_empty(&mut self, pos: &Size) -> bool {
        self.get_pos(pos).is_none()
    }
    pub fn is_valid_move(&mut self, pos: &Size) -> bool {
        self.is_empty(pos) && pos.x < self.size.x && pos.y < self.size.y
    }
    fn check_cache_integrity(&mut self) {
        if
            self.cache.len() !=
            (self.size.x * self.size.y).try_into().expect("Could not convert to usize")
        {
            self.init_cache();
        }
    }
    fn init_cache(&mut self) {
        self.cache = Vec::new();
        for _y in 0..self.size.y {
            for _x in 0..self.size.x {
                self.cache.push(None);
            }
        }
    }
    fn update_cache(&mut self, x: i32, y: i32, player: i32) {
        self.cache[
            <i32 as TryInto<usize>>
                ::try_into(y * self.size.x + x)
                .expect("Could not convert to usize")
        ] = Some(player);
    }
}
