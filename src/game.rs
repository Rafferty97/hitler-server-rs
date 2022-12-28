use self::action::ExecutiveAction;
use self::board::Board;
use self::deck::Deck;
use self::eligible::EligiblePlayers;
pub use self::options::GameOptions;
use self::party::Party;
use self::player::{assign_roles, Player, Role};
use self::votes::{MonarchistVotes, Votes};
use self::{confirmations::Confirmations, government::Government};
use crate::error::GameError;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

mod action;
mod board;
mod confirmations;
mod deck;
mod eligible;
mod government;
mod json;
mod options;
mod party;
mod player;
mod test;
mod votes;

pub const MAX_PLAYERS: usize = 16;

/// A game of Secret Hitler.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Game {
    opts: GameOptions,
    players: Vec<Player>,
    board: Board,
    deck: Deck,
    state: GameState,
    presidential_turn: usize,
    election_tracker: usize,
    last_government: Option<Government>,
    radicalised: bool,
    assassinated: bool,
    rng: rand_chacha::ChaCha8Rng,
}

/// Represents the current phase in the game loop.
#[derive(Clone, Serialize, Deserialize, Debug)]
enum GameState {
    Night {
        confirmations: Confirmations,
    },
    Election {
        president: usize,
        chancellor: Option<usize>,
        eligible_chancellors: EligiblePlayers,
        votes: Votes,
    },
    MonarchistElection {
        /// The player who is the monarchist, and therefore the next president
        monarchist: usize,
        /// The last president who unlocked the special election power
        president: usize,
        /// Whether the monarchist has elected to hijack the election.
        confirmed: bool,
        /// The monarchist's choice for chancellor
        monarchist_chancellor: Option<usize>,
        /// The last president's choice for chancellor
        president_chancellor: Option<usize>,
        /// Players eligible to be selected as chancellor
        eligible_chancellors: EligiblePlayers,
        /// Players votes for the chancellor; the monarchist breaks ties
        votes: MonarchistVotes,
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
        board_ready: bool,
    },
    CommunistStart {
        action: ExecutiveAction,
    },
    Congress,
    ChoosePlayer {
        action: ExecutiveAction,
        can_select: EligiblePlayers,
        can_be_selected: EligiblePlayers,
        communist_reveal: bool,
    },
    CommunistEnd {
        action: ExecutiveAction,
        chosen_player: Option<usize>,
    },
    ActionReveal {
        action: ExecutiveAction,
        chosen_player: Option<usize>,
        confirmations: Confirmations,
        communist_reveal: bool,
    },
    GameOver(WinCondition),
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
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

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
enum VetoStatus {
    CannotVeto,
    CanVeto,
    VetoDenied,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
enum WinCondition {
    /// The liberals completed their policy track.
    LiberalPolicyTrack,
    /// The fascists completed their policy track.
    FascistPolicyTrack,
    /// The communists completed their policy track.
    CommunistPolicyTrack,
    /// Hitler was elected chancellor
    HitlerChancellor,
    /// Hitler was executed
    HitlerExecuted,
    /// The Capitalist was executed
    CapitalistExecuted,
}

impl ToString for WinCondition {
    fn to_string(&self) -> String {
        match self {
            WinCondition::LiberalPolicyTrack => "LiberalPolicyTrack",
            WinCondition::FascistPolicyTrack => "FascistPolicyTrack",
            WinCondition::CommunistPolicyTrack => "CommunistPolicyTrack",
            WinCondition::HitlerChancellor => "HitlerChancellor",
            WinCondition::HitlerExecuted => "HitlerExecuted",
            WinCondition::CapitalistExecuted => "CapitalistExecuted",
        }
        .to_string()
    }
}

impl Game {
    /// Creates a new game of Secret Hitler.
    pub fn new(opts: GameOptions, player_names: &[String], seed: u64) -> Result<Self, GameError> {
        let num_players = player_names.len();

        // Generate the players and their roles
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let roles = assign_roles(num_players, &opts, &mut rng)?;
        let players = player_names
            .iter()
            .zip(roles)
            .map(|(name, role)| Player::new(name.into(), role))
            .collect::<Vec<_>>();

        // Create the board; shuffle the deck
        let board = Board::new(num_players);
        let mut deck = Deck::new(opts.communists);
        deck.shuffle(&board, &mut rng);

        // Return the new game
        Ok(Game {
            opts,
            players,
            board,
            deck,
            state: GameState::Night {
                confirmations: Confirmations::new(num_players),
            },
            presidential_turn: rng.gen_range(0..num_players),
            election_tracker: 0,
            last_government: None,
            radicalised: false,
            assassinated: false,
            rng,
        })
    }

    /// Gets the player names.
    pub fn player_names(&self) -> impl Iterator<Item = &'_ str> {
        self.players.iter().map(|p| &p.name[..])
    }

    /// Finds a player with the given name.
    pub fn find_player(&self, name: &str) -> Result<usize, GameError> {
        self.players
            .iter()
            .position(|p| p.name == name)
            .ok_or(GameError::PlayerNotFound)
    }

    /// Called when a player is ready to end the night round.
    pub fn end_night_round(&mut self, player: usize) -> Result<(), GameError> {
        self.check_player_index(player)?;
        let GameState::Night { confirmations } = &mut self.state else {
            return Err(GameError::InvalidAction);
        };
        let can_proceed = confirmations.confirm(player);
        if can_proceed {
            self.start_election(None);
        }
        Ok(())
    }

    /// Called when a player is ready to end the card reveal.
    pub fn end_card_reveal(&mut self, player: Option<usize>) -> Result<(), GameError> {
        let GameState::CardReveal { result, chaos, confirmations, board_ready } = &mut self.state else {
            return Err(GameError::InvalidAction);
        };

        if let Some(player) = player {
            confirmations.confirm(player);
        } else {
            *board_ready = true;
        }

        // Skip player confirmations if the game is over
        let players_ready = confirmations.can_proceed() || self.board.is_winning_card(*result);
        if !players_ready || !*board_ready {
            return Ok(());
        }

        // Play the card
        let (result, chaos) = (*result, *chaos);
        self.board.play_card(result);
        if self.check_game_over() {
            return Ok(());
        }
        self.check_deck();
        if let (false, Some(action)) = (chaos, self.board.get_executive_power(result)) {
            self.start_executive_action(action);
        } else {
            self.start_election(None);
        }
        Ok(())
    }

    /// Ends the legislative session.
    pub fn end_legislative_session(&mut self) -> Result<(), GameError> {
        let GameState::LegislativeSession { turn, .. } = &mut self.state else {
            return Err(GameError::InvalidAction);
        };
        let LegislativeSessionTurn::VetoApproved = turn else {
            return Err(GameError::InvalidAction);
        };
        self.election_tracker += 1;
        self.check_deck();
        self.start_election(None);
        Ok(())
    }

    /// Called when a player casts their vote.
    pub fn cast_vote(&mut self, player: usize, vote: bool) -> Result<(), GameError> {
        self.check_player_index(player)?;
        let GameState::Election {  chancellor, votes, .. } = &mut self.state else {
            return Err(GameError::InvalidAction);
        };
        if chancellor.is_none() {
            return Err(GameError::InvalidAction);
        }
        votes.vote(player, vote);
        Ok(())
    }

    /// Called when a player casts their vote in a monarchist election.
    pub fn cast_monarchist_vote(&mut self, player: usize, vote: usize) -> Result<(), GameError> {
        self.check_player_index(player)?;
        let GameState::MonarchistElection { monarchist_chancellor, president_chancellor, votes, .. } = &mut self.state else {
            return Err(GameError::InvalidAction);
        };
        let (Some(c1), Some(c2)) = (*monarchist_chancellor, *president_chancellor) else {
            return Err(GameError::InvalidAction);
        };
        if vote == c1 || vote == c2 {
            return Err(GameError::InvalidPlayerChoice);
        };
        votes.vote(player, vote == c1);
        Ok(())
    }

    /// Called when a player chooses another player.
    pub fn choose_player(&mut self, player: usize, other: usize) -> Result<(), GameError> {
        self.check_player_index(other)?;
        self.check_player_index(player)?;

        match &mut self.state {
            GameState::Election {
                president,
                chancellor,
                eligible_chancellors,
                ..
            } => {
                if player != *president || chancellor.is_some() {
                    return Err(GameError::InvalidAction);
                }
                if !eligible_chancellors.includes(other) {
                    return Err(GameError::InvalidPlayerChoice);
                }
                *chancellor = Some(other);
                Ok(())
            }
            GameState::ChoosePlayer {
                action,
                can_select,
                can_be_selected,
                communist_reveal,
            } => {
                use ExecutiveAction::*;
                if !can_select.includes(player) {
                    return Err(GameError::InvalidAction);
                }
                if !can_be_selected.includes(other) {
                    return Err(GameError::InvalidPlayerChoice);
                }
                let (action, communist_reveal) = (*action, *communist_reveal);
                match action {
                    InvestigatePlayer | SpecialElection | Execution | Confession => {
                        self.state = GameState::ActionReveal {
                            action,
                            chosen_player: Some(player),
                            confirmations: Confirmations::new(self.num_players_alive()),
                            communist_reveal,
                        };
                    }
                    Bugging | Radicalisation | Congress => {
                        self.state = GameState::CommunistEnd {
                            action,
                            chosen_player: Some(player),
                        };
                    }
                    _ => panic!("Invalid game state"),
                }
                Ok(())
            }
            GameState::MonarchistElection {
                monarchist,
                president,
                confirmed,
                monarchist_chancellor,
                president_chancellor,
                eligible_chancellors,
                ..
            } => {
                if !*confirmed {
                    return Err(GameError::InvalidAction);
                }
                let (turn, chancellor) = if monarchist_chancellor.is_none() {
                    (*monarchist, monarchist_chancellor)
                } else if president_chancellor.is_none() {
                    (*president, monarchist_chancellor)
                } else {
                    return Err(GameError::InvalidAction);
                };
                if player != turn || !eligible_chancellors.includes(other) {
                    return Err(GameError::InvalidPlayerChoice);
                }
                *chancellor = Some(other);
                eligible_chancellors.exclude(other);
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
            GameState::MonarchistElection {
                monarchist,
                monarchist_chancellor,
                president_chancellor,
                votes,
                ..
            } => {
                let (Some(c1), Some(c2)) = (*monarchist_chancellor, *president_chancellor) else {
                    return Err(GameError::InvalidAction);
                };
                let Some(outcome) = votes.outcome() else {
                    return Err(GameError::InvalidAction);
                };
                self.start_legislative_session(Government {
                    president: *monarchist,
                    chancellor: if outcome { c1 } else { c2 },
                });
                self.check_game_over();
                Ok(())
            }
            _ => Err(GameError::InvalidAction),
        }
    }

    /// Called when a player discards a policy from their hand.
    pub fn discard_policy(&mut self, player: usize, card_idx: usize) -> Result<(), GameError> {
        use LegislativeSessionTurn::*;

        self.check_player_index(player)?;

        let GameState::LegislativeSession { president, chancellor, turn } = &mut self.state else {
            return Err(GameError::InvalidAction);
        };

        match turn {
            President { cards } if player == *president => {
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
            Chancellor { cards, .. } if player == *chancellor => {
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
    pub fn veto_agenda(&mut self, player: usize) -> Result<(), GameError> {
        use LegislativeSessionTurn::*;

        self.check_player_index(player)?;

        let GameState::LegislativeSession { president, chancellor, turn } = &mut self.state else {
            return Err(GameError::InvalidAction);
        };

        match turn {
            Chancellor { cards, veto } => {
                if *veto == VetoStatus::CanVeto && player == *chancellor {
                    *turn = VetoRequested { cards: *cards };
                    Ok(())
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            VetoRequested { .. } => {
                if player == *president {
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
    pub fn reject_veto(&mut self, player: usize) -> Result<(), GameError> {
        self.check_player_index(player)?;

        let GameState::LegislativeSession { president, turn, .. } = &mut self.state else {
            return Err(GameError::InvalidAction);
        };

        let LegislativeSessionTurn::VetoRequested { cards } = turn else {
            return Err(GameError::InvalidAction);
        };

        if player != *president {
            return Err(GameError::InvalidAction);
        }

        *turn = LegislativeSessionTurn::Chancellor {
            cards: *cards,
            veto: VetoStatus::VetoDenied,
        };

        Ok(())
    }

    /// Returns true if the game is over.
    pub fn game_over(&self) -> bool {
        matches!(self.state, GameState::GameOver { .. })
    }

    /// Returns whether a particular player has won.
    pub fn player_has_won(&self, player: usize) -> bool {
        let GameState::GameOver(outcome) = self.state else {
            return false;
        };
        let player = &self.players[player];
        match outcome {
            WinCondition::LiberalPolicyTrack => player.party() == Party::Liberal,
            WinCondition::FascistPolicyTrack => player.party() == Party::Fascist,
            WinCondition::CommunistPolicyTrack => player.party() == Party::Communist,
            WinCondition::HitlerExecuted => !matches!(player.role, Role::Fascist | Role::Hitler),
            WinCondition::HitlerChancellor => matches!(player.role, Role::Fascist | Role::Hitler),
            WinCondition::CapitalistExecuted => player.party() == Party::Communist,
        }
    }

    fn start_election(&mut self, president: Option<usize>) {
        if self.election_tracker == 3 {
            let card = self.deck.draw_one();
            self.last_government = None;
            self.play_card(card, true);
            return;
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
        let cards = self.deck.draw_three();
        self.state = GameState::LegislativeSession {
            president: government.president,
            chancellor: government.chancellor,
            turn: LegislativeSessionTurn::President { cards },
        };
        self.last_government = Some(government);
    }

    fn play_card(&mut self, card: Party, chaos: bool) {
        self.state = GameState::CardReveal {
            result: card,
            chaos,
            confirmations: Confirmations::new(self.num_players_alive()),
            board_ready: false,
        };
        self.election_tracker = 0;
    }

    fn check_deck(&mut self) {
        self.deck.check_shuffle(&self.board, &mut self.rng);
    }

    fn check_game_over(&mut self) -> bool {
        // Check for legislative victory
        if let Some(party) = self.board.check_tracks() {
            self.state = GameState::GameOver(match party {
                Party::Liberal => WinCondition::LiberalPolicyTrack,
                Party::Fascist => WinCondition::FascistPolicyTrack,
                Party::Communist => WinCondition::CommunistPolicyTrack,
            });
            return true;
        }

        // Check whether Hitler was elected chancellor
        if self.board.fascist_cards >= 3 {
            if let GameState::LegislativeSession { chancellor, .. } = &self.state {
                let player = &mut self.players[*chancellor];
                if player.role == Role::Hitler {
                    self.state = GameState::GameOver(WinCondition::HitlerChancellor);
                    return true;
                } else {
                    player.not_hitler = true;
                }
            }
        }

        // Check whether Hitler has been executed
        if !self.hitler().alive {
            self.state = GameState::GameOver(WinCondition::HitlerExecuted);
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

    /// Gets the number of ordinary fascists in the game.
    pub fn num_ordinary_fascists(&self) -> usize {
        self.players
            .iter()
            .filter(|p| p.role == Role::Fascist)
            .count()
    }

    /// Returns `Ok` if the given player index is valid, and an `Err` otherwise.
    fn check_player_index(&self, player: usize) -> Result<(), GameError> {
        if player < self.num_players() {
            Ok(())
        } else {
            Err(GameError::InvalidPlayerIndex)
        }
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
    fn eligble_chancellors(&self, president: usize) -> EligiblePlayers {
        let mut result = self.eligible_players().exclude(president);

        if let Some(government) = self.last_government {
            result = result.exclude(government.chancellor);
            if self.num_players_alive() > 5 {
                result = result.exclude(government.president);
            }
        }

        result.make()
    }
}
