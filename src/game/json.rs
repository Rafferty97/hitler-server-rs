use super::{
    player::{Player, Role},
    Game, GameError,
};
use crate::game::VetoStatus;
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
        })
    }

    pub fn get_player_json(&self, name: &str) -> Result<Value, GameError> {
        let idx = self.get_player_idx(name)?;
        let player = &self.players[idx];

        let can_view_roles = match player.role {
            Role::Liberal => false,
            Role::Fascist => true,
            Role::Hitler => self.players.len() < 7,
        };

        Ok(json!({
            "id": player.name,
            "name": player.name,
            "role": player.role.to_string(),
            "action": self.get_player_action_json(player),
            "players": self.get_players_json(can_view_roles),
            "isDead": !player.alive
        }))
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
            } => json!({
                "type": "executiveAction",
                "action": match action {
                    Investigate => "investigate",
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

    fn get_player_action_json(&self, player: &Player) -> Value {
        Value::Null // FIXME
    }
}
