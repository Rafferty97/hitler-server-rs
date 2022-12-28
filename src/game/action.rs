use crate::game::{
    eligible::EligiblePlayers,
    executive_power::ExecutiveAction,
    player::Role,
    votes::{MonarchistVotes, Votes},
    LegislativeSessionTurn, VetoStatus,
};

use super::{party::Party, Game, GameState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum PlayerAction {
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
    Dead,
    // FIXME
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum ChoosePlayerKind {
    /// The player is selecting a chancellor nominee
    NominateChancellor,
    /// The player is selecting the next presidential nominee
    NominatePresident,
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
    pub fn get_player_action(&self, player: usize) -> Option<PlayerAction> {
        use GameState::*;

        let player_idx = player;
        let player = &self.players[player_idx];

        if !player.alive {
            return Some(PlayerAction::Dead);
        }

        match &self.state {
            Night { confirmations } => {
                (!confirmations.has_confirmed(player_idx)).then_some(PlayerAction::Night)
            }

            Election {
                president,
                chancellor,
                eligible_chancellors,
                votes,
            } => match chancellor {
                None => (player_idx == *president).then_some(PlayerAction::ChoosePlayer {
                    kind: ChoosePlayerKind::NominateChancellor,
                    options: eligible_chancellors.names(self),
                }),
                Some(_) => (!votes.has_cast(player_idx)).then_some(PlayerAction::Vote),
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
                    (player.role == Role::Monarchist).then_some(PlayerAction::HijackElection)
                } else if monarchist_chancellor.is_none() {
                    (player_idx == *monarchist).then_some(PlayerAction::ChoosePlayer {
                        kind: ChoosePlayerKind::NominateChancellor,
                        options: eligible_chancellors.names(self),
                    })
                } else if president_chancellor.is_none() {
                    (player_idx == *president).then_some(PlayerAction::ChoosePlayer {
                        kind: ChoosePlayerKind::NominateChancellor,
                        options: eligible_chancellors.names(self),
                    })
                } else {
                    (!votes.has_cast(player_idx)).then_some(PlayerAction::ChoosePlayer {
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
                    .then_some(PlayerAction::PresidentDisard { cards: *cards }),
                LegislativeSessionTurn::Chancellor { cards, veto } => (player_idx == *president)
                    .then_some(PlayerAction::ChancellorDiscard {
                        cards: *cards,
                        can_veto: *veto == VetoStatus::CanVeto,
                    }),
                LegislativeSessionTurn::VetoRequested { .. } => {
                    (player_idx == *president).then_some(PlayerAction::ApproveVeto)
                }
                LegislativeSessionTurn::VetoApproved => None,
            },

            CardReveal {
                confirmations,
                board_ready,
                ..
            } => (*board_ready && !confirmations.has_confirmed(player_idx)).then_some(
                PlayerAction::StartElection {
                    can_assassinate: !self.assassinated && player.role == Role::Anarchist,
                },
            ),

            CommunistStart { .. } => None,

            Congress { .. } => {
                (player.role == Role::Communist).then_some(PlayerAction::EndCongress)
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
                PlayerAction::ChoosePlayer {
                    kind,
                    options: can_be_selected.names(self),
                }
            }),

            _ => unimplemented!(), // FIXME
        }
    }
}

// ChoosePlayer {
// CommunistEnd {
// ActionReveal {
// GameOver(WinCondition),
