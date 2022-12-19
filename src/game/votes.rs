use serde::{Deserialize, Serialize};

/// Tracks the vote of each player.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Votes {
    num_players: usize,
    votes: [Option<bool>; 10],
}

impl Votes {
    /// Creates a new `Votes`.
    pub fn new(num_players: usize) -> Self {
        let votes = [None; 10];
        Self { num_players, votes }
    }

    /// Returns whether the given player has cast their vote.
    pub fn has_cast(&self, player_idx: usize) -> bool {
        self.votes[player_idx].is_some()
    }

    /// Records the vote of a player.
    pub fn vote(&mut self, player_idx: usize, vote: bool) {
        self.votes[player_idx] = Some(vote);
    }

    /// If all votes are counted, returns the outcome, otherwise returns `None`.
    pub fn outcome(&self) -> Option<bool> {
        let yes = self.votes.iter().filter(|v| **v == Some(true)).count();
        let no = self.votes.iter().filter(|v| **v == Some(false)).count();
        (yes + no >= self.num_players).then(|| yes > no)
    }

    /// Gets the votes of each player.
    pub fn votes(&self) -> &[Option<bool>] {
        &self.votes[..self.num_players]
    }
}
