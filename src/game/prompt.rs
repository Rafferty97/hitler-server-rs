use super::{government::Government, party::Party, Game, GameState, WinCondition};
use crate::game::{
    executive_power::ExecutiveAction, player::Role, LegislativeSessionTurn, VetoStatus,
};
use serde::{Deserialize, Serialize};

// #[derive(Clone, Serialize, Deserialize)]
// pub enum GameUpdate {
//     NoGame,
//     InvalidGame {
//         game_id: String,
//     },
//     Board {
//         game_id: String,
//         players: Vec<OtherPlayer>,
//         // FIXME: Boards, deck, etc.
//         prompt: Option<BoardPrompt>,
//     },
//     Player {
//         game_id: String,
//         players: Vec<OtherPlayer>,
//         prompt: Option<PlayerPrompt>,
//     },
// }

#[derive(Clone, Serialize, Deserialize)]
pub struct BoardUpdate {
    players: Vec<OtherPlayer>,
    election_tracker: usize,
    liberal_cards: usize,
    fascist_cards: usize,
    communist_cards: Option<usize>,
    draw_pile: usize,
    presidential_turn: usize,
    last_government: Option<Government>,
    prompt: Option<BoardPrompt>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerUpdate {
    players: Vec<OtherPlayer>,
    name: String,
    role: Role,
    prompt: Option<PlayerPrompt>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OtherPlayer {
    pub name: String,
    pub party: Option<Party>,
    pub role: Option<Role>,
    pub alive: bool,
    pub not_hitler: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum BoardPrompt {
    EndVoting,
    EndCardReveal,
    EndCommunistStart,
    EndCommunistEnd,
    EndExecutivePower,
    EndLegislativeSession,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PlayerPrompt {
    Lobby {
        can_start: bool,
    },
    Night,
    ChoosePlayer {
        kind: ChoosePlayerKind,
        options: Vec<String>,
    },
    Vote,
    HijackElection,
    PresidentDisard {
        cards: [Party; 3],
    },
    ChancellorDiscard {
        cards: [Party; 2],
        can_veto: bool,
    },
    ApproveVeto,
    StartElection {
        can_assassinate: bool,
    },
    EndCongress,
    EndExecutivePower,
    Dead,
    GameOver(WinCondition),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum ChoosePlayerKind {
    /// The player is selecting a chancellor nominee
    NominateChancellor,
    /// The player is selecting the next presidential nominee
    NominatePresident,
    /// The player is selecting the first chancellor in a monarchist election
    MonarchistFirstChancellor,
    /// The player is selecting the second chancellor in a monarchist election
    MonarchistSecondChancellor,
    /// The player is voting on a chancellor
    VoteChancellor,
    /// The player is choosing another player to investigate their party membership
    Investigate,
    /// The player is choosing another player to execute
    Execute,
    /// The player is choosing another player to attempt to convert to communism
    Radicalise,
    /// The player is choosing which player must reveal their party membership to all
    Confession,
}

impl Game {
    pub fn get_board_update(&self) -> BoardUpdate {
        BoardUpdate {
            players: self.get_players(None),
            election_tracker: self.election_tracker,
            liberal_cards: self.board.liberal_cards,
            fascist_cards: self.board.fascist_cards,
            communist_cards: self.opts.communists.then_some(self.board.communist_cards),
            draw_pile: self.deck.count(),
            presidential_turn: self.presidential_turn,
            last_government: self.last_government,
            prompt: self.get_board_prompt(), // FIXME: Add current state
        }
    }

    pub fn get_player_update(&self, player_idx: usize) -> PlayerUpdate {
        let player = &self.players[player_idx];
        PlayerUpdate {
            players: self.get_players(Some(player.role)),
            name: player.name,
            role: player.role,
            prompt: self.get_player_prompt(player_idx),
        }
    }

    pub fn get_players(&self, for_eyes: Option<Role>) -> Vec<OtherPlayer> {
        // FIXME: Reveal roles to certain players
        self.players
            .iter()
            .map(|player| OtherPlayer {
                name: player.name,
                party: None,
                role: None,
                alive: player.alive,
                not_hitler: player.not_hitler,
            })
            .collect()
    }

    pub fn get_board_prompt(&self) -> Option<BoardPrompt> {
        use GameState::*;

        match &self.state {
            Night { .. } => None,

            Election { votes, .. } => votes.outcome().map(|_| BoardPrompt::EndVoting),

            MonarchistElection { votes, .. } => votes.outcome().map(|_| BoardPrompt::EndVoting),

            LegislativeSession { turn, .. } => match turn {
                LegislativeSessionTurn::VetoApproved => Some(BoardPrompt::EndLegislativeSession),
                _ => None,
            },

            CardReveal { board_ready, .. } => (!board_ready).then_some(BoardPrompt::EndCardReveal),

            CommunistStart { .. } => Some(BoardPrompt::EndCommunistStart),

            Congress { .. } => None,

            ChoosePlayer { .. } => None,

            CommunistEnd { .. } => Some(BoardPrompt::EndCommunistEnd),

            ActionReveal { action, .. } => match action {
                ExecutiveAction::SpecialElection => Some(BoardPrompt::EndExecutivePower),
                ExecutiveAction::Execution => Some(BoardPrompt::EndExecutivePower),
                ExecutiveAction::FiveYearPlan => Some(BoardPrompt::EndExecutivePower),
                ExecutiveAction::Confession => Some(BoardPrompt::EndExecutivePower),
                ExecutiveAction::Assassination => Some(BoardPrompt::EndExecutivePower),
                _ => None,
            },

            GameOver(_) => None,
        }
    }

    pub fn get_player_prompt(&self, player: usize) -> Option<PlayerPrompt> {
        use GameState::*;

        let player_idx = player;
        let player = &self.players[player_idx];

        if !player.alive && !self.game_over() {
            return Some(PlayerPrompt::Dead);
        }

        match &self.state {
            Night { confirmations } => {
                (!confirmations.has_confirmed(player_idx)).then_some(PlayerPrompt::Night)
            }

            Election {
                president,
                chancellor,
                eligible_chancellors,
                votes,
            } => match chancellor {
                None => (player_idx == *president).then_some(PlayerPrompt::ChoosePlayer {
                    kind: ChoosePlayerKind::NominateChancellor,
                    options: eligible_chancellors.names(self),
                }),
                Some(_) => (!votes.has_cast(player_idx)).then_some(PlayerPrompt::Vote),
            },

            MonarchistElection {
                monarchist,
                president,
                confirmed,
                monarchist_chancellor,
                president_chancellor,
                eligible_chancellors,
                votes,
            } => {
                if !confirmed {
                    (player.role == Role::Monarchist).then_some(PlayerPrompt::HijackElection)
                } else if monarchist_chancellor.is_none() {
                    (player_idx == *monarchist).then_some(PlayerPrompt::ChoosePlayer {
                        kind: ChoosePlayerKind::MonarchistFirstChancellor,
                        options: eligible_chancellors.names(self),
                    })
                } else if president_chancellor.is_none() {
                    (player_idx == *president).then_some(PlayerPrompt::ChoosePlayer {
                        kind: ChoosePlayerKind::MonarchistSecondChancellor,
                        options: eligible_chancellors.names(self),
                    })
                } else {
                    (!votes.has_cast(player_idx)).then_some(PlayerPrompt::ChoosePlayer {
                        kind: ChoosePlayerKind::VoteChancellor,
                        options: [*monarchist_chancellor, *president_chancellor]
                            .into_iter()
                            .map(|i| self.players[i.unwrap()].name.clone())
                            .collect(),
                    })
                }
            }

            LegislativeSession {
                president,
                chancellor,
                turn,
            } => match turn {
                LegislativeSessionTurn::President { cards } => (player_idx == *president)
                    .then_some(PlayerPrompt::PresidentDisard { cards: *cards }),
                LegislativeSessionTurn::Chancellor { cards, veto } => (player_idx == *chancellor)
                    .then_some(PlayerPrompt::ChancellorDiscard {
                        cards: *cards,
                        can_veto: *veto == VetoStatus::CanVeto,
                    }),
                LegislativeSessionTurn::VetoRequested { .. } => {
                    (player_idx == *president).then_some(PlayerPrompt::ApproveVeto)
                }
                LegislativeSessionTurn::VetoApproved => None,
            },

            CardReveal {
                confirmations,
                board_ready,
                ..
            } => (*board_ready && !confirmations.has_confirmed(player_idx)).then_some(
                PlayerPrompt::StartElection {
                    can_assassinate: !self.assassinated && player.role == Role::Anarchist,
                },
            ),

            CommunistStart { .. } => None,

            Congress { .. } => {
                (player.role == Role::Communist).then_some(PlayerPrompt::EndCongress)
            }

            ChoosePlayer {
                action,
                can_select,
                can_be_selected,
            } => can_select.includes(player_idx).then(|| {
                use ExecutiveAction::*;
                let kind = match action {
                    InvestigatePlayer | Bugging => ChoosePlayerKind::Investigate,
                    SpecialElection => ChoosePlayerKind::NominatePresident,
                    Execution | Assassination => ChoosePlayerKind::Execute,
                    Radicalisation | Congress => ChoosePlayerKind::Radicalise,
                    Confession => ChoosePlayerKind::Confession,
                    PolicyPeak | FiveYearPlan => panic!("Invalid action"),
                };
                PlayerPrompt::ChoosePlayer {
                    kind,
                    options: can_be_selected.names(self),
                }
            }),

            CommunistEnd { .. } => None,

            ActionReveal {
                action,
                confirmations,
                ..
            } => {
                use ExecutiveAction::*;
                let government = self.last_government.unwrap();
                let show_button = match action {
                    InvestigatePlayer | PolicyPeak => player_idx == government.president,
                    Bugging | Radicalisation | Congress => !confirmations.has_confirmed(player_idx),
                    _ => false,
                };
                show_button.then_some(PlayerPrompt::EndExecutivePower)
            }

            GameOver(outcome) => Some(PlayerPrompt::GameOver(*outcome)),
        }
    }
}

// fn can_view_role(&self, player_idx: usize, other_idx: usize) -> bool {
//     use Role::*;
//     let player = &self.players[player_idx];
//     let other = &self.players[other_idx];

//     if player.role == Capitalist {
//         let to_left = (player_idx + 1) % self.num_players() == other_idx;
//         let to_right = (other_idx + 1) % self.num_players() == player_idx;
//         return to_left || to_right;
//     }

//     match (player.role, other.role) {
//         (Fascist, Fascist) => true,
//         (Fascist, Hitler) => true,
//         (Fascist, Monarchist) => true,
//         (Hitler, Fascist) => self.num_ordinary_fascists() < 2,
//         (Communist, Communist) => true,
//         (Communist, Anarchist) => true,
//         (Centrist, Centrist) => true,
//         _ => false,
//     }
// }
