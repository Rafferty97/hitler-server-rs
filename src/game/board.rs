use serde::{Deserialize, Serialize};

const MAX_LIBERAL_CARDS: usize = 5;
const MAX_FASCIST_CARDS: usize = 6;

#[derive(Clone, Serialize, Deserialize)]
pub struct Board {
    pub num_players: usize,
    pub liberal_cards: usize,
    pub fascist_cards: usize,
}

impl Board {
    /// Creates a new board.
    pub fn new(num_players: usize) -> Self {
        Board {
            num_players,
            liberal_cards: 0,
            fascist_cards: 0,
        }
    }

    /// Plays a liberal card, returning `true` if the liberals have filled their track.
    pub fn play_liberal(&mut self) -> bool {
        self.liberal_cards += 1;
        self.liberal_cards == MAX_LIBERAL_CARDS
    }

    /// Plays a fascist card, returning `true` if the fascists have filled their track.
    pub fn play_fascist(&mut self) -> bool {
        self.fascist_cards += 1;
        self.fascist_cards == MAX_FASCIST_CARDS
    }
}
