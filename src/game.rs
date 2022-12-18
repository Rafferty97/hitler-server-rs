use self::board::Board;
use self::party::{shuffle_deck, Party};
use self::player::{Player, Role, RoleAssigner};
use self::votes::Votes;
use self::{confirmations::Confirmations, government::Government};
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
    presidential_turn: usize,
    election_tracker: usize,
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
    /// This player cannot be chosen for this action.
    InvalidPlayerChoice,
    /// This action cannot be performed during this phase of the game.
    InvalidAction,
    /// An invalid card was chosen.
    InvalidCard,
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
        eligible_chancellors: [bool; 10],
        votes: Votes,
    },
    LegislativeSession {
        president: usize,
        chancellor: usize,
        turn: LegislativeSessionTurn,
    },
    CardReveal {
        result: Party,
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
    President { cards: [Party; 3] },
    /// Chancellor must discard a card.
    Chancellor { cards: [Party; 2], veto: VetoStatus },
    /// Chancellor has called for a veto.
    VetoRequested { cards: [Party; 2] },
    /// President has approved the veto.
    VetoApproved,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum VetoStatus {
    CannotVeto,
    CanVeto,
    VetoDenied,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
        if !(5..=10).contains(&num_players) {
            panic!("Must have at 5-10 players in a game.");
        }

        // Generate the players and their roles
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let roles = RoleAssigner::new(num_players, &mut rng);
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
            presidential_turn: rng.gen_range(0..num_players),
            election_tracker: 0,
            last_government: None,
            rng,
        }
    }

    /// Called when a player clicks the "next" button.
    pub fn player_next(&mut self, name: &str) -> Result<(), GameError> {
        let idx = self.get_player_idx(name)?;
        match &mut self.state {
            GameState::Night { confirmations } => {
                let can_proceed = confirmations.confirm(idx);
                if can_proceed {
                    self.start_election(None);
                }
                Ok(())
            }
            GameState::CardReveal {
                result,
                confirmations,
                chaos,
            } => {
                let can_proceed = confirmations.confirm(idx);
                if can_proceed {
                    let (result, chaos) = (*result, *chaos);
                    self.end_card_reveal(result, chaos);
                }
                Ok(())
            }
            _ => Err(GameError::InvalidAction),
        }
    }

    /// Called when the board is ready to move on.
    pub fn board_next(&mut self) -> Result<(), GameError> {
        match &self.state {
            GameState::LegislativeSession { turn, .. } => {
                if let LegislativeSessionTurn::VetoApproved { .. } = turn {
                    self.election_tracker += 1;
                    self.start_election(None);
                    Ok(())
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            GameState::CardReveal { result, chaos, .. } => {
                // Skip player confirmations if the game is over
                if self.board.is_winning_card(*result) {
                    self.end_card_reveal(*result, *chaos);
                }
                Ok(())
            }
            _ => Err(GameError::InvalidAction),
        }
    }

    /// Called when a player casts their vote.
    pub fn cast_vote(&mut self, name: &str, vote: bool) -> Result<(), GameError> {
        let idx = self.get_player_idx(name)?;
        match &mut self.state {
            GameState::Election {
                chancellor, votes, ..
            } => {
                if chancellor.is_none() {
                    return Err(GameError::InvalidAction);
                }
                votes.vote(idx, vote);
                Ok(())
            }
            _ => Err(GameError::InvalidAction),
        }
    }

    /// Called when a player chooses another player.
    pub fn choose_player(&mut self, name: &str, other: &str) -> Result<(), GameError> {
        let own_idx = self.get_player_idx(name)?;
        let other_idx = self.get_player_idx(other)?;
        if own_idx == other_idx {
            return Err(GameError::InvalidPlayerChoice);
        }

        match &mut self.state {
            GameState::Election {
                chancellor,
                eligible_chancellors,
                ..
            } => {
                if chancellor.is_some() {
                    return Err(GameError::InvalidAction);
                }
                if !eligible_chancellors[other_idx] {
                    return Err(GameError::InvalidPlayerChoice);
                }
                *chancellor = Some(other_idx);
                Ok(())
            }
            GameState::ExecutiveAction {
                action,
                player_chosen,
            } => {
                if player_chosen.is_some() {
                    return Err(GameError::InvalidAction);
                }
                if *action == ExecutiveAction::PolicyPeak {
                    return Err(GameError::InvalidAction);
                }
                if !self.players[other_idx].alive {
                    return Err(GameError::InvalidPlayerChoice);
                }
                *player_chosen = Some(other_idx);
                Ok(())
            }
            _ => Err(GameError::InvalidAction),
        }
    }

    /// Called when the board has finished revealing the election result.
    pub fn end_voting(&mut self) -> Result<(), GameError> {
        match &self.state {
            GameState::Election {
                president,
                chancellor,
                votes,
                ..
            } => {
                let Some(chancellor) = chancellor else {
                    return Err(GameError::InvalidAction);
                };
                let Some(passed) = votes.outcome() else {
                    return Err(GameError::InvalidAction);
                };
                let government = Government {
                    president: *president,
                    chancellor: *chancellor,
                };
                if passed {
                    self.start_legislative_session(government);
                    self.check_game_over();
                } else {
                    self.election_tracker += 1;
                    self.start_election(None);
                }
                Ok(())
            }
            _ => Err(GameError::InvalidAction),
        }
    }

    /// Called when a player discards a policy from their hand.
    pub fn discard_policy(&mut self, name: &str, card_idx: usize) -> Result<(), GameError> {
        use LegislativeSessionTurn::*;

        let idx = self.get_player_idx(name)?;

        let GameState::LegislativeSession { president, chancellor, turn } = &mut self.state else {
            return Err(GameError::InvalidAction);
        };

        match turn {
            President { cards } if idx == *president => {
                let cards = match card_idx {
                    0 => [cards[1], cards[2]],
                    1 => [cards[0], cards[2]],
                    2 => [cards[0], cards[1]],
                    _ => return Err(GameError::InvalidCard),
                };
                *turn = Chancellor {
                    cards,
                    veto: if self.board.veto_unlocked() {
                        VetoStatus::CanVeto
                    } else {
                        VetoStatus::CannotVeto
                    },
                };
            }
            Chancellor { cards, .. } if idx == *chancellor => {
                let card = match card_idx {
                    0 => cards[1],
                    1 => cards[0],
                    _ => return Err(GameError::InvalidCard),
                };
                self.play_card(card, false);
            }
            _ => return Err(GameError::InvalidAction),
        }

        Ok(())
    }

    /// Called when the chancellor proposes a veto, or the president consents to a proposed veto.
    pub fn veto_agenda(&mut self, name: &str) -> Result<(), GameError> {
        use LegislativeSessionTurn::*;

        let idx = self.get_player_idx(name)?;

        let GameState::LegislativeSession { president, chancellor, turn } = &mut self.state else {
            return Err(GameError::InvalidAction);
        };

        match turn {
            Chancellor { cards, veto } => {
                if *veto == VetoStatus::CanVeto && idx == *chancellor {
                    *turn = VetoRequested { cards: *cards };
                    Ok(())
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            VetoRequested { .. } => {
                if idx == *president {
                    *turn = VetoApproved;
                    Ok(())
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            _ => Err(GameError::InvalidAction),
        }
    }

    /// Called when the president rejects a proposed veto.
    pub fn reject_veto(&mut self, name: &str) -> Result<(), GameError> {
        let idx = self.get_player_idx(name)?;

        let GameState::LegislativeSession { president, chancellor, turn } = &mut self.state else {
            return Err(GameError::InvalidAction);
        };

        let LegislativeSessionTurn::VetoRequested { cards } = turn else {
            return Err(GameError::InvalidAction);
        };

        if idx != *president {
            return Err(GameError::InvalidAction);
        }

        *turn = LegislativeSessionTurn::Chancellor {
            cards: *cards,
            veto: VetoStatus::VetoDenied,
        };

        Ok(())
    }

    /// Called when the board has finished presenting the executive action.
    pub fn end_executive_action(&mut self) -> Result<(), GameError> {
        let GameState::ExecutiveAction { action, player_chosen } = &mut self.state else {
            return Err(GameError::InvalidAction);
        };

        match action {
            ExecutiveAction::Execution => {
                let Some(idx) = player_chosen else {
                    return Err(GameError::InvalidAction);
                };
                let player = &mut self.players[*idx];
                player.alive = false;
                player.not_hitler = player.role != Role::Hitler;
                self.check_game_over();
            }
            ExecutiveAction::SpecialElection => {
                let Some(idx) = player_chosen else {
                    return Err(GameError::InvalidAction);
                };
                let president = Some(*idx);
                self.start_election(president);
            }
            _ => {
                self.start_election(None);
            }
        }

        Ok(())
    }

    fn start_election(&mut self, president: Option<usize>) {
        if self.election_tracker == 3 {
            let card = self.deck.pop().unwrap();
            self.last_government = None;
            self.play_card(card, true);
        }

        let president = president.unwrap_or_else(|| {
            self.presidential_turn = self.next_player(self.presidential_turn);
            self.presidential_turn
        });

        self.state = GameState::Election {
            president,
            chancellor: None,
            eligible_chancellors: self.eligble_chancellors(president),
            votes: Votes::new(self.num_players_alive()),
        };
    }

    fn start_legislative_session(&mut self, government: Government) {
        self.state = GameState::LegislativeSession {
            president: government.president,
            chancellor: government.chancellor,
            turn: LegislativeSessionTurn::President {
                cards: [
                    self.deck.pop().unwrap(),
                    self.deck.pop().unwrap(),
                    self.deck.pop().unwrap(),
                ],
            },
        };
        self.last_government = Some(government);
    }

    fn play_card(&mut self, card: Party, chaos: bool) {
        self.state = GameState::CardReveal {
            result: card,
            chaos,
            confirmations: Confirmations::new(self.num_players_alive()),
        };
        self.election_tracker = 0;
    }

    fn end_card_reveal(&mut self, result: Party, chaos: bool) {
        match result {
            Party::Liberal => {
                self.board.play_liberal();
                self.check_deck();
                self.check_game_over();
            }
            Party::Fascist => {
                self.board.play_fascist();
                self.check_deck();
                if self.check_game_over() {
                    return;
                }
                if !chaos {
                    self.play_executive_power();
                }
            }
        }
    }

    fn play_executive_power(&mut self) {
        if let Some(action) = self.board.get_executive_power() {
            self.state = GameState::ExecutiveAction {
                action,
                player_chosen: None,
            };
        } else {
            self.start_election(None);
        }
    }

    fn check_deck(&mut self) {
        if self.deck.len() < 3 {
            self.deck = shuffle_deck(&self.board);
        }
    }

    fn check_game_over(&mut self) -> bool {
        // Check for legislative victory
        if let Some(party) = self.board.check_tracks() {
            self.state = GameState::GameOver {
                winner: party,
                win_condition: WinCondition::Legislative,
            };
            return true;
        }

        // Check whether Hitler was elected chancellor
        if self.board.fascist_cards >= 3 {
            if let GameState::LegislativeSession { chancellor, .. } = &self.state {
                if self.players[*chancellor].role == Role::Hitler {
                    self.state = GameState::GameOver {
                        winner: Party::Fascist,
                        win_condition: WinCondition::Hitler,
                    };
                    return true;
                }
            }
        }

        // Check whether Hitler has been executed
        if !self.hitler().alive {
            self.state = GameState::GameOver {
                winner: Party::Liberal,
                win_condition: WinCondition::Hitler,
            };
            return true;
        }

        false
    }

    /// Gets the number of players in the game.
    pub fn num_players(&self) -> usize {
        self.players.len()
    }

    /// Gets the number of players in the game that are alive.
    pub fn num_players_alive(&self) -> usize {
        self.players.iter().filter(|p| p.alive).count()
    }

    /// Gets the index of the player with the given name.
    fn get_player_idx(&self, name: &str) -> Result<usize, GameError> {
        self.players
            .iter()
            .position(|p| p.name == name)
            .ok_or(GameError::PlayerNotFound)
    }

    /// Finds the next alive player.
    fn next_player(&self, player: usize) -> usize {
        (player + 1..self.num_players())
            .chain(0..player)
            .find(|idx| self.players[*idx].alive)
            .unwrap()
    }

    /// Gets the player who is Hitler.
    fn hitler(&self) -> &Player {
        self.players
            .iter()
            .find(|player| player.role == Role::Hitler)
            .unwrap()
    }

    /// Determines which players are eligble to be chancellor.
    fn eligble_chancellors(&self, president: usize) -> [bool; 10] {
        let mut result = [false; 10];

        // Dead players are ineligble
        for (index, player) in self.players.iter().enumerate() {
            result[index] = player.alive;
        }

        // Chancellor must not be the president
        result[president] = false;

        // Last chancellor, and sometimes last president are ineligble
        if let Some(government) = self.last_government {
            result[government.chancellor] = false;
            if self.num_players_alive() > 5 {
                result[government.president] = false;
            }
        }

        result
    }
}
