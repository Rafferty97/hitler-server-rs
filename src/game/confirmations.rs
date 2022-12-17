use serde::{Deserialize, Serialize};

/// Tracks the acknowledgement status of each player,
/// such that game play can only proceed once all players have elected to move on.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Confirmations {
    num_players: usize,
    state: [bool; 10],
}

impl Confirmations {
    /// Creates a new `Confirmations`.
    pub fn new(num_players: usize) -> Self {
        let state = [false; 10];
        Self { num_players, state }
    }

    /// Records the acknowledgement of a player, and returns `true` iff the game can now proceed.
    pub fn confirm(&mut self, player_idx: usize) -> bool {
        self.state[player_idx] = true;
        self.state.iter().take(self.num_players).all(|c| *c)
    }
}
