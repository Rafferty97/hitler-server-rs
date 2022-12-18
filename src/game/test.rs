#![cfg(test)]

use crate::game::Game;

#[test]
fn can_create_game() {
    let players = ["Alex", "Bob", "Charlie", "David", "Ed"].map(|s| s.into());
    Game::new(&players, 0);
}
