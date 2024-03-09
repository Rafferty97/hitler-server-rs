use super::{party::Party, player::Role, Game, MAX_PLAYERS};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct EligiblePlayers {
    eligible: [bool; MAX_PLAYERS],
}

impl EligiblePlayers {
    pub fn only_one(player: usize) -> Self {
        Self {
            eligible: core::array::from_fn(|i| i == player),
        }
    }

    pub fn only(players: &[usize]) -> Self {
        Self {
            eligible: core::array::from_fn(|i| players.contains(&i)),
        }
    }

    pub fn exclude(&mut self, player: usize) {
        self.eligible[player] = false;
    }

    pub fn includes(&self, player: usize) -> bool {
        self.eligible[player]
    }

    pub fn names(&self, game: &Game) -> Vec<String> {
        game.players
            .iter()
            .enumerate()
            .filter(|(i, _)| self.includes(*i))
            .map(|(_, p)| p.name.clone())
            .collect()
    }
}

pub struct EligiblePlayersBuilder<'a> {
    game: &'a Game,
    eligible: [bool; MAX_PLAYERS],
}

impl Game {
    pub fn eligible_players(&self) -> EligiblePlayersBuilder<'_> {
        EligiblePlayersBuilder {
            game: self,
            eligible: core::array::from_fn(|i| self.players.get(i).map(|p| p.alive).unwrap_or(false)),
        }
    }
}

impl<'a> EligiblePlayersBuilder<'a> {
    pub fn exclude(mut self, player: usize) -> Self {
        self.eligible[player] = false;
        self
    }

    pub fn ordinary_communist(mut self) -> Self {
        for (idx, player) in self.game.players.iter().enumerate() {
            self.eligible[idx] &= player.role == Role::Communist;
        }
        self
    }

    pub fn can_radicalise(mut self) -> Self {
        for (idx, player) in self.game.players.iter().enumerate() {
            self.eligible[idx] &= player.party() != Party::Communist;
            self.eligible[idx] &= !player.investigated;
            self.eligible[idx] &= !player.tried_to_radicalise;
        }
        self
    }

    pub fn not_investigated(mut self) -> Self {
        for (idx, player) in self.game.players.iter().enumerate() {
            self.eligible[idx] &= !player.investigated;
        }
        self
    }

    pub fn make(self) -> EligiblePlayers {
        EligiblePlayers { eligible: self.eligible }
    }
}
