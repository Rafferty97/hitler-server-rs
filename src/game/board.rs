use super::{executive_power::ExecutiveAction, party::Party};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Board {
    pub num_players: usize,
    pub liberal_cards: usize,
    pub fascist_cards: usize,
    pub communist_cards: usize,
}

impl Board {
    /// Creates a new board.
    pub fn new(num_players: usize) -> Self {
        Board {
            num_players,
            liberal_cards: 0,
            fascist_cards: 0,
            communist_cards: 0,
        }
    }

    /// Plays a policy card.
    pub fn play_card(&mut self, party: Party) {
        match party {
            Party::Liberal => self.liberal_cards += 1,
            Party::Fascist => self.fascist_cards += 1,
            Party::Communist => self.communist_cards += 1,
        }
    }

    /// Gets the executive action unlocked by the last played fascist card, if there is any.
    pub fn get_executive_power(&self, party: Party) -> Option<ExecutiveAction> {
        use ExecutiveAction::*;
        match party {
            Party::Liberal => None,
            Party::Fascist => match (self.num_players, self.fascist_cards) {
                (9..=10, 1) => Some(InvestigatePlayer),
                (7..=10, 2) => Some(InvestigatePlayer),
                (5..=6, 3) => Some(PolicyPeak),
                (7..=10, 3) => Some(SpecialElection),
                (_, 4) => Some(Execution),
                (_, 5) => Some(Execution),
                _ => None,
            },
            Party::Communist => match (self.num_players, self.communist_cards) {
                (_, 1) => Some(Bugging),
                (_, 2) => Some(Radicalisation),
                (_, 3) => Some(FiveYearPlan),
                (_, 4) => Some(Congress),
                (8.., 5) => Some(Confession),
                _ => None,
            },
        }
    }

    /// Checks whether the card about to be played wins the game.
    pub fn is_winning_card(&self, party: Party) -> bool {
        match party {
            Party::Liberal => self.liberal_cards == self.max_liberal_cards() - 1,
            Party::Fascist => self.fascist_cards == self.max_fascist_cards() - 1,
            Party::Communist => self.communist_cards == self.max_communist_cards() - 1,
        }
    }

    /// Checks whether either party has completed their policy track.
    pub fn check_tracks(&self) -> Option<Party> {
        if self.liberal_cards == self.max_liberal_cards() {
            return Some(Party::Liberal);
        }
        if self.fascist_cards == self.max_fascist_cards() {
            return Some(Party::Fascist);
        }
        if self.communist_cards == self.max_communist_cards() {
            return Some(Party::Communist);
        }
        None
    }

    /// Checks whether veto power is unlocked.
    pub fn veto_unlocked(&self) -> bool {
        self.fascist_cards >= 5
    }

    fn max_liberal_cards(&self) -> usize {
        5
    }

    fn max_fascist_cards(&self) -> usize {
        6
    }

    fn max_communist_cards(&self) -> usize {
        if self.num_players < 8 {
            5
        } else {
            6
        }
    }
}
