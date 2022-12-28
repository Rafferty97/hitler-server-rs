use super::{party::Party, player::Role, Game, MAX_PLAYERS};

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

    pub fn includes(&self, player: usize) -> bool {
        self.eligible[player]
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
            eligible: core::array::from_fn(|i| {
                self.players.get(i).map(|p| p.alive).unwrap_or(false)
            }),
        }
    }
}

impl<'a> EligiblePlayersBuilder<'a> {
    pub fn exclude(self, player: usize) -> Self {
        self.eligible[player] = false;
        self
    }

    pub fn ordinary_communist(self) -> Self {
        for (idx, player) in self.game.players.iter().enumerate() {
            self.eligible[idx] &= player.role == Role::Communist;
        }
    }

    pub fn not_communist(self) -> Self {
        for (idx, player) in self.game.players.iter().enumerate() {
            self.eligible[idx] &= player.party() != Party::Communist;
        }
    }

    pub fn not_investigated(self) -> Self {
        for (idx, player) in self.game.players.iter().enumerate() {
            self.eligible[idx] &= !player.investigated;
        }
    }

    pub fn make(self) -> EligiblePlayers {
        EligiblePlayers {
            eligible: self.eligible,
        }
    }
}
