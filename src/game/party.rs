use serde::{Deserialize, Serialize};

use super::board::Board;

/// The two political parties of the game.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Party {
    Liberal,
    Fascist,
}

/// Gets a shuffled deck of cards, excluding those already on the board.
pub fn shuffle_deck(board: &Board) -> Vec<Party> {
    vec![] // FIXME
}
