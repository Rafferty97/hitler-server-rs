use serde::{Deserialize, Serialize};

/// Tracks the acknowledgement status of each player,
/// such that game play can only proceed once all players have elected to move on.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Confirmations {
    num_players: usize,
    state: [bool; 10],
}

impl Confirmations {
    /// Creates a new `Confirmations`,
    /// where `num_players` is the number of confirmations needed to proceed.
    pub fn new(num_players: usize) -> Self {
        let state = [false; 10];
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
        self.state.iter().filter(|c| **c).count() >= self.num_players
    }
}
