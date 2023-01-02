use super::{
    player::{Player, Role},
    Game, WinCondition,
};
use crate::{
    game::{party::Party, GameState, LegislativeSessionTurn, VetoStatus},
    pg::Outcome,
};
use serde_json::{json, Value};

impl Game {
    pub fn get_board_json(&self) -> Value {
        json!({
            "players": self.get_players_json(false),
            "state": self.get_board_state_json(),
            "electionTracker": self.election_tracker,
            "numLiberalCards": self.board.liberal_cards,
            "numFascistCards": self.board.fascist_cards,
            "drawPile": self.deck.len(),
            "lastPresident": self.last_government.map(|g| g.president as i32).unwrap_or(-1),
            "lastChancellor": self.last_government.map(|g| g.chancellor as i32).unwrap_or(-1),
            "hidden": self.get_hidden_state_json()
        })
    }

    pub fn get_lobby_board_json(players: &[String]) -> Value {
        json!({
            "players": Self::get_lobby_players_json(players),
            "state": { "type": "lobby" },
            "electionTracker": 0,
            "numLiberalCards": 0,
            "numFascistCards": 0,
            "drawPile": 0,
            "lastPresident": -1,
            "lastChancellor": -1,
        })
    }

    pub fn get_player_json(&self, player: usize) -> Value {
        let idx = player;
        let player = &self.players[idx];

        let can_view_roles = match player.role {
            Role::Liberal => false,
            Role::Fascist => true,
            Role::Hitler => self.players.len() < 7,
        };

        json!({
            "id": player.name,
            "name": player.name,
            "role": player.role.to_string(),
            "action": self.get_player_action_json(idx, player),
            "players": self.get_players_json(can_view_roles),
            "isDead": !player.alive
        })
    }

    pub fn get_lobby_player_json(names: &[String], player: usize) -> Value {
        json!({
            "id": names[player],
            "name": names[player],
            "action": {
                "type": "lobby",
                "canStart": names.len() >= 5
            },
            "players": Self::get_lobby_players_json(names),
            "isDead": false
        })
    }

    fn get_players_json(&self, include_roles: bool) -> Value {
        self.players
            .iter()
            .map(|player| {
                json!({
                    "id": player.name,
                    "name": player.name,
                    "isDead": !player.alive,
                    "isConfirmedNotHitler": player.not_hitler,
                    "hasBeenInvestigated": player.investigated,
                    "role": include_roles.then_some(player.role)
                })
            })
            .collect()
    }

    pub fn get_lobby_players_json(players: &[String]) -> Value {
        players
            .iter()
            .map(|name| {
                json!({
                    "id": name,
                    "name": name,
                    "isDead": false,
                    "isConfirmedNotHitler": false,
                    "hasBeenInvestigated": false
                })
            })
            .collect()
    }

    pub fn outcome(&self) -> Option<Outcome> {
        let GameState::GameOver { winner, win_condition, .. } = &self.state else {
            return None;
        };
        Some(match (winner, win_condition) {
            (Party::Liberal, WinCondition::Legislative) => Outcome::LiberalPolicyTrack,
            (Party::Liberal, WinCondition::Hitler) => Outcome::HitlerExecuted,
            (Party::Fascist, WinCondition::Legislative) => Outcome::FascistPolicyTrack,
            (Party::Fascist, WinCondition::Hitler) => Outcome::HitlerChancellor,
        })
    }

    fn get_board_state_json(&self) -> Value {
        use super::ExecutiveAction::*;
        use super::GameState::*;
        use super::LegislativeSessionTurn::*;

        match &self.state {
            Night { .. } => json!({ "type": "nightRound" }),
            Election {
                president,
                chancellor,
                votes,
                ..
            } => json!({
                "type": "election",
                "presidentElect": president,
                "chancellorElect": chancellor,
                "votes": votes.votes(),
                "voteResult": votes.outcome()
            }),
            LegislativeSession {
                president,
                chancellor,
                turn,
            } => json!({
                "type": "legislativeSession",
                "president": president,
                "chancellor": chancellor,
                "turn": match turn {
                    President { .. } => "President",
                    Chancellor { veto, .. } => match veto {
                        VetoStatus::VetoDenied => "VetoRejected",
                        _ => "Chancellor"
                    },
                    VetoRequested { .. } => "Veto",
                    VetoApproved => "VetoApproved"
                }
            }),
            CardReveal { result, chaos, .. } => json!({
                "type": "cardReveal",
                "card": result.to_string(),
                "chaos": chaos
            }),
            ExecutiveAction {
                action,
                player_chosen,
                ..
            } => json!({
                "type": "executiveAction",
                "action": match action {
                    InvestigatePlayer => "investigate",
                    SpecialElection => "specialElection",
                    PolicyPeak => "policyPeak",
                    Execution => "execution"
                },
                "playerChosen": player_chosen
            }),
            GameOver {
                winner,
                win_condition,
            } => json!({
                "type": "end",
                "winner": winner.to_string(),
                "winType": win_condition.to_string()
            }),
        }
    }

    fn get_hidden_state_json(&self) -> Value {
        json!({
            "players": serde_json::to_value(&self.players).unwrap(),
            "deck": serde_json::to_value(&self.deck).unwrap(),
            "hand": match &self.state {
                GameState::LegislativeSession { turn, .. } => {
                    match turn {
                        LegislativeSessionTurn::President { cards } => serde_json::to_value(cards).unwrap(),
                        LegislativeSessionTurn::Chancellor { cards, .. } => serde_json::to_value(cards).unwrap(),
                        LegislativeSessionTurn::VetoRequested { cards, .. } => serde_json::to_value(cards).unwrap(),
                        _ => Value::Null
                    }
                },
                _ => Value::Null
            }
        })
    }

    fn get_player_action_json(&self, idx: usize, player: &Player) -> Value {
        use super::GameState::*;

        // Dead players can't perform any actions
        if !player.alive {
            return Value::Null;
        }

        match &self.state {
            Night { confirmations } if !confirmations.has_confirmed(idx) => {
                json!({ "type": "nightRound" })
            }
            Election {
                president,
                chancellor,
                eligible_chancellors,
                votes,
            } => {
                if let Some(chancellor) = chancellor {
                    if !votes.has_cast(idx) {
                        return json!({
                            "type": "vote",
                            "president": president,
                            "chancellor": chancellor
                        });
                    }
                } else if idx == *president {
                    return json!({
                        "type": "choosePlayer",
                        "subtype": "nominateChancellor",
                        "players": (0..self.num_players())
                            .filter(|i| eligible_chancellors[*i])
                            .collect::<Value>(),
                    });
                }
                Value::Null
            }
            LegislativeSession {
                president,
                chancellor,
                turn,
            } => {
                use super::LegislativeSessionTurn::*;
                match turn {
                    President { cards } if idx == *president => json!({
                        "type": "legislative",
                        "role": "President",
                        "cards": cards.map(|c| c.to_string()),
                        "canVeto": false
                    }),
                    Chancellor { cards, veto } if idx == *chancellor => json!({
                        "type": "legislative",
                        "role": "Chancellor",
                        "cards": cards.map(|c| c.to_string()),
                        "canVeto": *veto == super::VetoStatus::CanVeto
                    }),
                    VetoRequested { .. } if idx == *president => json!({
                        "type": "vetoConsent",
                        "chancellor": chancellor
                    }),
                    _ => Value::Null,
                }
            }
            CardReveal {
                confirmations,
                board_ready,
                ..
            } => {
                if *board_ready && !confirmations.has_confirmed(idx) {
                    json!({ "type": "nextRound" })
                } else {
                    Value::Null
                }
            }
            ExecutiveAction {
                president,
                action,
                player_chosen,
                eligible_players,
            } => {
                use super::ExecutiveAction::*;
                if idx != *president {
                    return Value::Null;
                }
                match (*action, player_chosen) {
                    (InvestigatePlayer, Some(other)) => json!({
                        "type": "investigateParty",
                        "player": other,
                        "party": self.players[*other].party().to_string(),
                    }),
                    (Execution | InvestigatePlayer | SpecialElection, None) => json!({
                        "type": "choosePlayer",
                        "subtype": action.to_string(),
                        "players": (0..self.num_players())
                            .filter(|i| eligible_players[*i])
                            .collect::<Value>()
                    }),
                    (PolicyPeak, None) => json!({
                        "type": "policyPeak",
                        "cards": self.deck[self.deck.len() - 3..]
                            .iter()
                            .map(Party::to_string)
                            .collect::<Value>()
                    }),
                    _ => Value::Null,
                }
            }
            GameOver {
                winner,
                win_condition,
            } => {
                json!({
                    "type": "gameover",
                    "winner": winner.to_string(),
                    "winType": win_condition.to_string()
                })
            }
            _ => Value::Null,
        }
    }
}
