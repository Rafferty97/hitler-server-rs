use self::board::Board;
use self::party::{shuffle_deck, LegislationResult, Party};
use self::player::{Player, RoleAssigner};
use self::votes::Votes;
use self::{confirmations::Confirmations, government::Government};
use arrayvec::ArrayVec;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

mod board;
mod confirmations;
mod government;
mod party;
mod player;
mod test;
mod votes;

/// A game of Secret Hitler.
#[derive(Clone, Serialize, Deserialize)]
pub struct Game {
    players: Vec<Player>,
    board: Board,
    deck: Vec<Party>,
    state: GameState,
    last_government: Option<Government>,
    rng: rand_chacha::ChaCha8Rng,
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
#[derive(Clone, Serialize, Deserialize)]
enum GameState {
    Night {
        confirmations: Confirmations,
    },
    Election {
        president: usize,
        chancellor: Option<usize>,
        votes: Votes,
        election_tracker: usize,
        is_special: bool,
    },
    LegislativeSession {
        president: usize,
        chancellor: usize,
        turn: LegislativeSessionTurn,
        cards: ArrayVec<Party, 3>,
    },
    CardReveal {
        result: LegislationResult,
        chaos: bool,
        confirmations: Confirmations,
    },
    ExecutiveAction {
        action: ExecutiveAction,
        player_chosen: Option<usize>,
    },
    GameOver {
        winner: Party,
        win_condition: WinCondition,
    },
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum LegislativeSessionTurn {
    /// President must discard a card.
    President,
    /// Chancellor must discard a card.
    Chancellor,
    /// Chancellor has called for a veto.
    Veto,
    /// President has denied the veto.
    VetoDenied,
    /// President has approved the veto.
    VetoApproved,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum ExecutiveAction {
    /// The president must investigate a player's loyalty.
    InvestigatePlayer,
    /// The president must call a special election.
    SpecialElection,
    /// The president must peek at the top three cards on the deck.
    PolicyPeak,
    /// The president must execute a player.
    Execution,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum WinCondition {
    /// The winning team completed their policy track.
    Legislative,
    /// The winning team either installed or assasinated Hitler.
    Hitler,
}

impl Game {
    /// Creates a new game of Secret Hitler.
    pub fn new<'a>(player_names: &[String], seed: u64) -> Self {
        // Check the number of players
        let num_players = player_names.len();
        if num_players < 5 || num_players > 10 {
            panic!("Must have at 5-10 players in a game.");
        }

        // Generate the players and their roles
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let mut roles = RoleAssigner::new(num_players, &mut rng);
        let players = player_names
            .iter()
            .enumerate()
            .map(|(i, name)| Player::new(name.into(), roles.get(i)))
            .collect::<Vec<_>>();

        // Create the board; shuffle the deck
        let board = Board::new(num_players);
        let deck = shuffle_deck(&board);

        // Return the new game
        Game {
            players,
            board,
            deck,
            state: GameState::Night {
                confirmations: Confirmations::new(num_players),
            },
            last_government: None,
            rng,
        }
    }

    /// Called when a player clicks the "next" button.
    pub fn player_acknowledge(&mut self, name: &str) -> Result<(), GameError> {
        let idx = self.get_player_idx(name)?;
        match &mut self.state {
            GameState::Night { confirmations } => {
                let can_proceed = confirmations.confirm(idx);
                if can_proceed {
                    self.start_election();
                }
                Ok(())
            }
            GameState::CardReveal { confirmations, .. } => {
                let can_proceed = confirmations.confirm(idx);
                if can_proceed {
                    self.end_card_reveal()
                } else {
                    Ok(())
                }
            }
            _ => Err(GameError::InvalidAction),
        }
    }

    /// Called when a player casts their vote.
    pub fn player_vote(&mut self, name: &str, vote: bool) -> Result<(), GameError> {
        unimplemented!()
    }

    fn start_election(&mut self) {
        let president = match &self.state {
            GameState::Night { .. } => self.random_player(),
            _ => panic!("Cannot start election"),
        };
        let election_tracker = match &self.state {
            GameState::Election {
                election_tracker, ..
            } => election_tracker + 1,
            _ => 0,
        };
        self.state = GameState::Election {
            president,
            chancellor: None,
            votes: Votes::new(self.num_players()),
            election_tracker,
            is_special: false, // FIXME
        }
    }

    fn end_card_reveal(&mut self) -> Result<(), GameError> {
        let GameState::CardReveal { result, chaos, confirmations } = self.state else {
            return Err(GameError::InvalidAction)
        };

        // FIXME

        Ok(())
    }

    /// Gets the number of players in the game.
    pub fn num_players(&self) -> usize {
        self.players.len()
    }

    /// Gets the index of the player with the given name.
    fn get_player_idx(&self, name: &str) -> Result<usize, GameError> {
        self.players
            .iter()
            .position(|p| p.name == name)
            .ok_or(GameError::PlayerNotFound)
    }

    /// Gets a random player.
    fn random_player(&mut self) -> usize {
        self.rng.gen_range(0..self.num_players())
    }

    /// Finds the next alive player.
    fn next_player(&self, player: usize) -> usize {
        (player + 1..self.num_players())
            .chain(0..player)
            .find(|idx| self.players[*idx].alive)
            .unwrap()
    }
}
