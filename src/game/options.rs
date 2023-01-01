use super::player::PlayerDistribution;
use crate::error::GameError;
use serde::{Deserialize, Serialize};

/// Options for customising the game of Secret Hitler or Secret Hitler XL.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
pub struct GameOptions {
    /// Whether to include the communists in the game.
    pub communists: bool,
    /// Whether to include the monarchist (fascist team).
    pub monarchist: bool,
    /// Whether to include the anarchist (communist team).
    pub anarchist: bool,
    /// Whether to include the capitalist (liberal team).
    pub capitalist: bool,
    /// Whether to include the centrists (liberal team).
    pub centrists: bool,
}

impl GameOptions {
    /// Gets the player distribution for this configuration for the given number of players.
    /// Returns a `GameError` if the combination of settings and player count is not valid.
    pub fn player_distribution(&self, num_players: usize) -> Result<PlayerDistribution, GameError> {
        PlayerDistribution::new(self, num_players)
    }

    /// Returns the minimum number of players for this configuration, or `None` if the configuration is not valid.
    pub fn min_players(&self) -> Option<usize> {
        (0..20).find(|num_players| self.player_distribution(*num_players).is_ok())
    }

    /// Returns the maxmimum number of players for this configuration, or `None` if the configuration is not valid.
    pub fn max_players(&self) -> Option<usize> {
        (0..20).rfind(|num_players| self.player_distribution(*num_players).is_ok())
    }
}
