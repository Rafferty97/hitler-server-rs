use super::{party::Party, ExecutiveAction};
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

    /// Plays a liberal card.
    pub fn play_liberal(&mut self) {
        self.liberal_cards += 1;
    }

    /// Plays a fascist card.
    pub fn play_fascist(&mut self) {
        self.fascist_cards += 1;
    }

    /// Gets the executive action unlocked by the last played fascist card, if there is any.
    pub fn get_executive_power(&self) -> Option<ExecutiveAction> {
        use ExecutiveAction::*;
        match (self.num_players, self.fascist_cards) {
            (9..=10, 1) => Some(InvestigatePlayer),
            (7..=10, 2) => Some(InvestigatePlayer),
            (5..=6, 3) => Some(PolicyPeak),
            (7..=10, 3) => Some(SpecialElection),
            (_, 4) => Some(Execution),
            (_, 5) => Some(Execution),
            _ => None,
        }
    }

    /// Checks whether the card about to be played wins the game.
    pub fn is_winning_card(&self, party: Party) -> bool {
        match party {
            Party::Liberal => self.liberal_cards == MAX_LIBERAL_CARDS - 1,
            Party::Fascist => self.fascist_cards == MAX_FASCIST_CARDS - 1,
        }
    }

    /// Checks whether either party has completed their policy track.
    pub fn check_tracks(&self) -> Option<Party> {
        if self.liberal_cards == MAX_LIBERAL_CARDS {
            return Some(Party::Liberal);
        }
        if self.fascist_cards == MAX_FASCIST_CARDS {
            return Some(Party::Fascist);
        }
        None
    }

    /// Checks whether veto power is unlocked.
    pub fn veto_unlocked(&self) -> bool {
        self.fascist_cards >= 5
    }
}
