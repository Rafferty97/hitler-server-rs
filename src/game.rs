use self::confirmations::Confirmations;
use self::votes::Votes;
use serde::{Deserialize, Serialize};

mod confirmations;
mod votes;

/// A game of Secret Hitler.
#[derive(Clone, Serialize, Deserialize)]
pub struct Game {
    players: Vec<Player>,
    state: GameState,
}

/// The result of attempting to perform an invalid operation on a [Game].
pub enum GameError {
    /// Not enough players to start the game.
    NotEnoughPlayers,
    /// No more players may join the game.
    TooManyPlayers,
    /// A player with the same name has already joined.
    PlayerWithSameName,
    /// No player exists with the given name.
    PlayerNotFound,
    /// This action cannot be performed during this phase of the game.
    InvalidAction,
}

/// Represents the current phase in the game loop.
#[derive(Clone, Copy, Serialize, Deserialize)]
enum GameState {
    Lobby,
    Night {
        confirmations: Confirmations,
    },
    Election {
        president: usize,
        chancellor: Option<usize>,
        votes: Votes,
        // presidentElect: number;
        // chancellorElect?: number;
        // votes: (boolean | null)[];
        // voteResult: boolean | null;
        // isSpecial: boolean;
    },
}

/// A game player.
#[derive(Clone, Serialize, Deserialize)]
struct Player {
    name: String,
}

impl Game {
    pub fn new() -> Self {
        Game {
            players: vec![],
            state: GameState::Lobby,
        }
    }

    pub fn add_player(&mut self, name: String) -> Result<(), GameError> {
        if self.players.len() == 10 {
            return Err(GameError::TooManyPlayers);
        }

        if self.players.iter().any(|p| p.name == name) {
            return Err(GameError::PlayerWithSameName);
        }

        self.players.push(Player::new(name));
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), GameError> {
        if self.players.len() < 5 {
            return Err(GameError::NotEnoughPlayers);
        }

        Ok(())
    }

    pub fn player_acknowledge(&mut self, name: &str) -> Result<(), GameError> {
        let idx = self.get_player_idx(name)?;
        match &mut self.state {
            GameState::Night { confirmations } => {
                if confirmations.confirm(idx) {
                    self.state = GameState::Election {
                        president: 0, // FIXME
                        chancellor: None,
                        votes: Votes::new(self.players.len()),
                    }
                }
                Ok(())
            }
            _ => Err(GameError::InvalidAction),
        }
    }

    fn get_player_idx(&self, name: &str) -> Result<usize, GameError> {
        self.players
            .iter()
            .position(|p| p.name == name)
            .ok_or(GameError::PlayerNotFound)
    }
}

impl Player {
    fn new(name: String) -> Self {
        Player { name }
    }
}
