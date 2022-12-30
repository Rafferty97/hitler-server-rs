use super::{
    government::Government, party::Party, player::InvestigationResult, Game, GameState,
    WinCondition,
};
use crate::game::{
    executive_power::ExecutiveAction, player::Role, AssassinationState, LegislativeSessionTurn,
    VetoStatus,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BoardUpdate {
    pub election_tracker: usize,
    pub liberal_cards: usize,
    pub fascist_cards: usize,
    pub communist_cards: Option<usize>,
    pub draw_pile: usize,
    pub presidential_turn: usize,
    pub last_government: Option<Government>,
    pub prompt: Option<BoardPrompt>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PlayerUpdate {
    pub name: String,
    pub role: Role,
    pub others: Vec<InvestigationResult>,
    pub prompt: Option<PlayerPrompt>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PublicPlayer {
    pub name: String,
    pub alive: bool,
    pub not_hitler: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum BoardPrompt {
    Night,
    Election {
        president: usize,
        chancellor: Option<usize>,
        votes: Vec<Option<bool>>,
        outcome: Option<bool>,
    },
    SpecialElection {
        monarchist_passed: bool,
        chosen_player: Option<usize>,
    },
    MonarchistElection {
        monarchist: usize,
        president: usize,
        monarchist_chancellor: Option<usize>,
        president_chancellor: Option<usize>,
        votes: Vec<Option<bool>>,
        outcome: Option<bool>,
    },
    LegislativeSession {
        president: usize,
        chancellor: usize,
        phase: LegislativePhase,
    },
    CardReveal {
        result: Party,
        chaos: bool,
        can_end: bool,
    },
    InvestigatePlayer {
        chosen_player: Option<usize>,
    },
    PolicyPeak,
    Execution {
        chosen_player: Option<usize>,
    },
    CommunistSession {
        action: ExecutiveAction,
        phase: CommunistSessionPhase,
    },
    FiveYearPlan,
    Confession {
        chosen_player: Option<usize>,
        party: Option<Party>,
    },
    Assassination {
        anarchist: usize,
        chosen_player: Option<usize>,
    },
    GameOver {
        outcome: WinCondition,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum PlayerPrompt {
    Night,
    ChoosePlayer {
        kind: ChoosePlayerKind,
        options: Vec<String>,
    },
    Vote,
    HijackElection,
    PresidentDiscard {
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
    InvestigatePlayer {
        name: String,
        party: Party,
    },
    PolicyPeak {
        cards: [Party; 3],
    },
    EndExecutivePower,
    Dead,
    GameOver {
        outcome: WinCondition,
    },
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
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

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum LegislativePhase {
    President,
    Chancellor,
    VetoRequested,
    VetoApproved,
    VetoRejected,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum CommunistSessionPhase {
    Entering,
    InProgress,
    Leaving,
    Reveal,
}

impl Game {
    pub fn get_board_update(&self) -> BoardUpdate {
        BoardUpdate {
            election_tracker: self.election_tracker,
            liberal_cards: self.board.liberal_cards,
            fascist_cards: self.board.fascist_cards,
            communist_cards: self.opts.communists.then_some(self.board.communist_cards),
            draw_pile: self.deck.count(),
            presidential_turn: self.presidential_turn,
            last_government: self.last_government,
            prompt: self.get_board_prompt(),
        }
    }

    pub fn get_player_update(&self, player_idx: usize) -> PlayerUpdate {
        let player = &self.players[player_idx];
        PlayerUpdate {
            name: player.name.clone(),
            role: player.role,
            others: player.others[..self.num_players()].to_vec(),
            prompt: self.get_player_prompt(player_idx),
        }
    }

    pub fn get_public_players(&self) -> Vec<PublicPlayer> {
        self.players
            .iter()
            .map(|player| PublicPlayer {
                name: player.name.clone(),
                alive: player.alive,
                not_hitler: player.not_hitler,
            })
            .collect()
    }

    pub fn get_board_prompt(&self) -> Option<BoardPrompt> {
        use GameState::*;

        match &self.state {
            Night { .. } => Some(BoardPrompt::Night),

            Election {
                president,
                chancellor,
                votes,
                ..
            } => Some(BoardPrompt::Election {
                president: *president,
                chancellor: *chancellor,
                votes: votes.votes().to_vec(),
                outcome: votes.outcome(),
            }),

            MonarchistElection {
                monarchist,
                president,
                confirmed,
                monarchist_chancellor,
                president_chancellor,
                votes,
                ..
            } => {
                if !confirmed {
                    Some(BoardPrompt::SpecialElection {
                        monarchist_passed: false,
                        chosen_player: None,
                    })
                } else {
                    Some(BoardPrompt::MonarchistElection {
                        monarchist: *monarchist,
                        president: *president,
                        monarchist_chancellor: *monarchist_chancellor,
                        president_chancellor: *president_chancellor,
                        votes: votes.votes().to_vec(),
                        outcome: votes.outcome(),
                    })
                }
            }

            LegislativeSession {
                president,
                chancellor,
                turn,
            } => Some(BoardPrompt::LegislativeSession {
                president: *president,
                chancellor: *chancellor,
                phase: match turn {
                    LegislativeSessionTurn::Chancellor { veto, .. } => match veto {
                        VetoStatus::CanVeto => LegislativePhase::Chancellor,
                        VetoStatus::CannotVeto => LegislativePhase::Chancellor,
                        VetoStatus::VetoDenied => LegislativePhase::VetoRejected,
                    },
                    LegislativeSessionTurn::President { .. } => LegislativePhase::President,
                    LegislativeSessionTurn::VetoRequested { .. } => LegislativePhase::VetoRejected,
                    LegislativeSessionTurn::VetoApproved => LegislativePhase::VetoApproved,
                },
            }),

            CardReveal {
                result,
                chaos,
                board_ready,
                ..
            } => Some(BoardPrompt::CardReveal {
                result: *result,
                chaos: *chaos,
                can_end: !*board_ready,
            }),

            CommunistStart { action } => Some(BoardPrompt::CommunistSession {
                action: *action,
                phase: CommunistSessionPhase::Entering,
            }),

            Congress { .. } => Some(BoardPrompt::CommunistSession {
                action: ExecutiveAction::Congress,
                phase: CommunistSessionPhase::InProgress,
            }),

            ChoosePlayer { action, .. } => Some(match action {
                ExecutiveAction::InvestigatePlayer => BoardPrompt::InvestigatePlayer {
                    chosen_player: None,
                },
                ExecutiveAction::SpecialElection => BoardPrompt::SpecialElection {
                    monarchist_passed: true,
                    chosen_player: None,
                },
                ExecutiveAction::Execution => BoardPrompt::Execution {
                    chosen_player: None,
                },
                ExecutiveAction::Bugging
                | ExecutiveAction::Radicalisation
                | ExecutiveAction::Congress => BoardPrompt::CommunistSession {
                    action: *action,
                    phase: CommunistSessionPhase::InProgress,
                },
                ExecutiveAction::FiveYearPlan => BoardPrompt::FiveYearPlan,
                ExecutiveAction::Confession => BoardPrompt::Confession {
                    chosen_player: None,
                    party: None,
                },
                _ => panic!("Invalid action"),
            }),

            CommunistEnd { action, .. } => Some(BoardPrompt::CommunistSession {
                action: *action,
                phase: CommunistSessionPhase::Leaving,
            }),

            ActionReveal {
                action,
                chosen_player,
                ..
            } => Some(match action {
                ExecutiveAction::InvestigatePlayer => BoardPrompt::InvestigatePlayer {
                    chosen_player: *chosen_player,
                },
                ExecutiveAction::SpecialElection => BoardPrompt::SpecialElection {
                    monarchist_passed: true,
                    chosen_player: *chosen_player,
                },
                ExecutiveAction::PolicyPeak => BoardPrompt::PolicyPeak,
                ExecutiveAction::Execution => BoardPrompt::Execution {
                    chosen_player: *chosen_player,
                },
                ExecutiveAction::Bugging
                | ExecutiveAction::Radicalisation
                | ExecutiveAction::Congress => BoardPrompt::CommunistSession {
                    action: *action,
                    phase: CommunistSessionPhase::Reveal,
                },
                ExecutiveAction::FiveYearPlan => BoardPrompt::FiveYearPlan,
                ExecutiveAction::Confession => BoardPrompt::Confession {
                    chosen_player: *chosen_player,
                    party: chosen_player.map(|i| self.players[i].party()),
                },
            }),

            Assassination {
                anarchist,
                chosen_player,
            } => Some(BoardPrompt::Assassination {
                anarchist: *anarchist,
                chosen_player: *chosen_player,
            }),

            GameOver(outcome) => Some(BoardPrompt::GameOver { outcome: *outcome }),
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
                    .then_some(PlayerPrompt::PresidentDiscard { cards: *cards }),
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
            } => {
                use AssassinationState::Unused;
                let unconfirmed = *board_ready && !confirmations.has_confirmed(player_idx);
                unconfirmed.then_some(PlayerPrompt::StartElection {
                    can_assassinate: self.assassination == Unused && player.role == Role::Anarchist,
                })
            }

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
                    Execution => ChoosePlayerKind::Execute,
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
                chosen_player,
            } => {
                use ExecutiveAction::*;
                let government = self.last_government.unwrap();
                match action {
                    InvestigatePlayer if player_idx == government.president => {
                        let player = &self.players[chosen_player.unwrap()];
                        Some(PlayerPrompt::InvestigatePlayer {
                            name: player.name.clone(),
                            party: player.party(),
                        })
                    }
                    PolicyPeak if player_idx == government.president => {
                        Some(PlayerPrompt::PolicyPeak {
                            cards: self.deck.peek_three(),
                        })
                    }
                    Bugging | Radicalisation | Congress => {
                        let waiting = !confirmations.has_confirmed(player_idx);
                        waiting.then_some(PlayerPrompt::EndExecutivePower)
                    }
                    _ => None,
                }
            }

            Assassination { chosen_player, .. } => {
                let anarchist = player.role == Role::Anarchist && chosen_player.is_none();
                anarchist.then_some(PlayerPrompt::ChoosePlayer {
                    kind: ChoosePlayerKind::Execute,
                    options: self
                        .eligible_players()
                        .exclude(player_idx)
                        .make()
                        .names(self),
                })
            }

            GameOver(outcome) => Some(PlayerPrompt::GameOver { outcome: *outcome }),
        }
    }
}
