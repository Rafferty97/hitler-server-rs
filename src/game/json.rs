use super::{
    player::{Player, Role},
    Game,
};
use crate::game::{party::Party, GameState};
use serde_json::{json, Value};

impl Game {
    pub fn get_board_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
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

        json!({
            "id": player.name,
            "name": player.name,
            "role": player.role.to_string(),
            "action": self.get_player_action_json(idx, player),
            "players": self.get_players_json(Some(idx)),
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

    fn get_players_json(&self, viewer: Option<usize>) -> Value {
        self.players
            .iter()
            .enumerate()
            .map(|(index, player)| {
                let view_role = match viewer {
                    Some(i) => self.can_view_role(i, index),
                    None => false,
                };
                json!({
                    "id": player.name,
                    "name": player.name,
                    "isDead": !player.alive,
                    "isConfirmedNotHitler": player.not_hitler,
                    "hasBeenInvestigated": player.investigated,
                    "role": view_role.then_some(player.role)
                })
            })
            .collect()
    }

    fn can_view_role(&self, player_idx: usize, other_idx: usize) -> bool {
        use Role::*;
        let player = &self.players[player_idx];
        let other = &self.players[other_idx];

        if player.role == Capitalist {
            let to_left = (player_idx + 1) % self.num_players() == other_idx;
            let to_right = (other_idx + 1) % self.num_players() == player_idx;
            return to_left || to_right;
        }

        match (player.role, other.role) {
            (Fascist, Fascist) => true,
            (Fascist, Hitler) => true,
            (Fascist, Monarchist) => true,
            (Hitler, Fascist) => self.num_ordinary_fascists() < 2,
            (Communist, Communist) => true,
            (Communist, Anarchist) => true,
            (Centrist, Centrist) => true,
            _ => false,
        }
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

    pub fn get_outcome_json(&self) -> Value {
        let GameState::GameOver(outcome) = &self.state else {
            return json!({ "finished": false });
        };
        json!({
            "finished": true,
            "outcome": outcome.to_string()
        })
    }

    fn get_board_state_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
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
                            .filter(|i| eligible_chancellors.includes(*i))
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
            GameOver(outcome) => {
                json!({
                    "type": "gameover",
                    "outcome": outcome.to_string()
                })
            }
            _ => Value::Null,
        }
    }
}
