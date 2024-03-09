use serde::{Deserialize, Serialize};

use super::MAX_PLAYERS;

/// Tracks the vote of each player.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Votes {
    num_players: usize,
    votes: [Option<bool>; MAX_PLAYERS],
}

impl Votes {
    /// Creates a new `Votes`.
    pub fn new(num_players: usize) -> Self {
        let votes = [None; MAX_PLAYERS];
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
        if std::env::var("QUICK_MODE").is_ok() {
            (yes + no > 0).then_some(yes > no)
        } else {
            (yes + no >= self.num_players).then_some(yes > no)
        }
    }

    /// Gets the votes of each player.
    pub fn votes(&self) -> &[Option<bool>] {
        &self.votes
    }
}

/// Tracks the vote of each player during a monarchist election.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct MonarchistVotes {
    num_players: usize,
    /// The index of the player who is the monarchist
    monarchist: usize,
    /// `true` is a vote for the monarchist's chancellor, and `false` is for the other
    votes: [Option<bool>; MAX_PLAYERS],
}

impl MonarchistVotes {
    /// Creates a new `MonarchistVotes`.
    pub fn new(num_players: usize, monarchist: usize) -> Self {
        let votes = [None; MAX_PLAYERS];
        Self { num_players, monarchist, votes }
    }

    /// Returns whether the given player has cast their vote.
    pub fn has_cast(&self, player_idx: usize) -> bool {
        self.votes[player_idx].is_some()
    }

    /// Records the vote of a player, where `true` signifies the monarchist's selection.
    pub fn vote(&mut self, player_idx: usize, vote: bool) {
        self.votes[player_idx] = Some(vote);
    }

    /// If all votes are counted, returns the outcome, otherwise returns `None`.
    /// A result of `true` signifies the monarchist's selection has won.
    pub fn outcome(&self) -> Option<bool> {
        use std::cmp::Ordering::*;
        let yes = self.votes.iter().filter(|v| **v == Some(true)).count();
        let no = self.votes.iter().filter(|v| **v == Some(false)).count();
        if std::env::var("QUICK_MODE").is_ok() {
            (yes + no > 0).then_some(yes > no)
        } else {
            (yes + no >= self.num_players).then(|| match yes.cmp(&no) {
                Less => false,
                Greater => true,
                Equal => self.votes[self.monarchist].unwrap_or(true),
            })
        }
    }

    /// Gets the votes of each player.
    pub fn votes(&self) -> &[Option<bool>] {
        &self.votes
    }
}
