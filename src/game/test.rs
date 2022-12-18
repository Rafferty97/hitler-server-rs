#![cfg(test)]

use super::GameState;
use crate::game::Game;

#[test]
fn can_create_game() {
    let players = ["Alex", "Bob", "Charlie", "David", "Ed"].map(|s| s.into());
    let game = Game::new(&players, 0);
    assert!(matches!(game.state, GameState::Night { .. }));
}
