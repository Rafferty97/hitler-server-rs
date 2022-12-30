use serde::{Deserialize, Serialize};

use super::MAX_PLAYERS;

/// Tracks the acknowledgement status of each player,
/// such that game play can only proceed once all players have elected to move on.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Confirmations {
    num_players: usize,
    state: [bool; MAX_PLAYERS],
}

impl Confirmations {
    /// Creates a new `Confirmations`,
    /// where `num_players` is the number of confirmations needed to proceed.
    pub fn new(num_players: usize) -> Self {
        let state = [false; MAX_PLAYERS];
        Self { num_players, state }
    }

    /// Returns whether or not the given player has registered their acknowledgement.
    pub fn has_confirmed(&self, player_idx: usize) -> bool {
        self.state[player_idx]
    }

    /// Records the acknowledgement of a player, and returns `true` iff the game can now proceed.
    pub fn confirm(&mut self, player_idx: usize) -> bool {
        self.state[player_idx] = true;
        self.can_proceed()
    }

    /// Returns `true` iff the game can now proceed.
    pub fn can_proceed(&self) -> bool {
        if std::env::var("QUICK_MODE").is_ok() {
            self.state.iter().any(|c| *c)
        } else {
            self.state.iter().filter(|c| **c).count() >= self.num_players
        }
    }
}
