use super::board::Board;
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};

/// The two political parties of the game.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Party {
    Liberal,
    Fascist,
}

impl ToString for Party {
    fn to_string(&self) -> String {
        match self {
            Party::Liberal => "Liberal",
            Party::Fascist => "Fascist",
        }
        .to_string()
    }
}

/// Gets a shuffled deck of cards, excluding those already on the board.
pub fn shuffle_deck(board: &Board, rng: &mut impl Rng) -> Vec<Party> {
    let liberals = 6 - board.liberal_cards;
    let fascists = 11 - board.fascist_cards;

    let mut deck = (0..liberals)
        .map(|_| Party::Liberal)
        .chain((0..fascists).map(|_| Party::Fascist))
        .collect::<Vec<_>>();

    deck.shuffle(rng);
    deck
}
