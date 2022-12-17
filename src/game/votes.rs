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

    /// Records the vote of a player, and return
    pub fn vote(&mut self, player_idx: usize, vote: bool) -> bool {
        self.votes[player_idx] = Some(vote);
        self.votes
            .iter()
            .take(self.num_players)
            .all(|c| c.is_some())
    }

    /// If all votes are counted, returns the outcome, otherwise returns `None`.
    pub fn outcome(&self) -> Option<bool> {
        let yes = self.votes.iter().filter(|v| **v == Some(true)).count();
        let no = self.votes.iter().filter(|v| **v == Some(false)).count();
        (yes + no >= self.num_players).then(|| yes > no)
    }
}
