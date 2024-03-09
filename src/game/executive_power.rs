use super::{player::Role, Game, GameState, NextPresident};
use crate::{
    error::GameError,
    game::{confirmations::Confirmations, eligible::EligiblePlayers, government::Government},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum ExecutiveAction {
    /// The president must investigate a player's loyalty.
    InvestigatePlayer,
    /// The president must call a special election.
    SpecialElection,
    /// The president must peek at the top three cards on the deck.
    PolicyPeak,
    /// The president must execute a player.
    Execution,
    /// The communists learn the party membership of one player.
    Bugging,
    /// The communists attempt to convert one player to the communist team.
    Radicalisation,
    /// 2 communist and 1 liberal policy are shuffled into the deck.
    FiveYearPlan,
    /// The new communist learns who their allies are, or another radicalisation is attempted.
    Congress,
    /// The president or chancellor reveals their party membership.
    Confession,
}

impl ToString for ExecutiveAction {
    fn to_string(&self) -> String {
        match self {
            ExecutiveAction::InvestigatePlayer => "investigate",
            ExecutiveAction::SpecialElection { .. } => "specialElection",
            ExecutiveAction::PolicyPeak => "policyPeak",
            ExecutiveAction::Execution => "execution",
            ExecutiveAction::Bugging => "bugging",
            ExecutiveAction::Radicalisation => "radicalisation",
            ExecutiveAction::FiveYearPlan => "fiveYearPlan",
            ExecutiveAction::Congress => "congress",
            ExecutiveAction::Confession => "confession",
        }
        .to_string()
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum ConfessionChoice {
    President,
    Chancellor,
}

impl Game {
    /// Begins an executive action.
    pub fn start_executive_action(&mut self, action: ExecutiveAction) {
        use ExecutiveAction::*;

        // There must have been a last government for an executive power to be played
        let Government { president, chancellor } = self.last_government.unwrap();

        match action {
            InvestigatePlayer => {
                self.state = GameState::ChoosePlayer {
                    action,
                    can_select: EligiblePlayers::only_one(president),
                    can_be_selected: self.eligible_players().not_investigated().exclude(president).make(),
                };
            }
            SpecialElection => {
                // Note: We still want to "wait for the monarchist" even if they're dead,
                // so as to not reveal any information to other players.
                let monarchist = self.players.iter().position(|p| p.role == Role::Monarchist);
                if let Some(monarchist) = monarchist {
                    self.state = GameState::PromptMonarchist {
                        monarchist,
                        last_president: president,
                        hijacked: false,
                    };
                } else {
                    self.state = GameState::ChoosePlayer {
                        action,
                        can_select: EligiblePlayers::only_one(president),
                        can_be_selected: self.eligible_players().exclude(president).make(),
                    };
                }
            }
            Execution => {
                self.state = GameState::ChoosePlayer {
                    action,
                    can_select: EligiblePlayers::only_one(president),
                    can_be_selected: self.eligible_players().exclude(president).make(),
                };
            }
            PolicyPeak | FiveYearPlan => {
                self.state = GameState::ActionReveal {
                    action,
                    chosen_player: None,
                    confirmations: Confirmations::new(self.num_players_alive()),
                };
            }
            Bugging | Radicalisation | Congress => {
                self.state = GameState::CommunistStart { action };
            }
            Confession => {
                self.state = GameState::ChoosePlayer {
                    action,
                    can_select: EligiblePlayers::only_one(chancellor),
                    can_be_selected: EligiblePlayers::only(&[president, chancellor]),
                };
            }
        }
    }

    /// Called when the board has finished entering a "communist session".
    pub fn end_communist_start(&mut self) -> Result<(), GameError> {
        use ExecutiveAction::*;

        let GameState::CommunistStart { action } = self.state else {
            return Err(GameError::InvalidAction);
        };

        // If radicalisation succeeded, there's no second attempt during congress
        if action == Congress && self.radicalised {
            self.state = GameState::Congress;
            return Ok(());
        }

        let can_select = self.eligible_players().ordinary_communist().make();

        let mut can_be_selected = self.eligible_players().can_radicalise();
        if matches!(action, Radicalisation | Congress) {
            can_be_selected = can_be_selected.not_investigated();
        }
        let can_be_selected = can_be_selected.make();

        self.state = GameState::ChoosePlayer { action, can_select, can_be_selected };
        Ok(())
    }

    /// Called when a player is ready to end the congress session.
    pub fn end_congress(&mut self, player: usize) -> Result<(), GameError> {
        let GameState::Congress = &self.state else {
            return Err(GameError::InvalidAction);
        };
        if self.players.get(player).map(|p| p.role) != Some(Role::Communist) {
            return Err(GameError::InvalidAction);
        }
        self.state = GameState::CommunistEnd {
            action: ExecutiveAction::Congress,
            chosen_player: None,
        };
        Ok(())
    }

    /// Called when the monarchist elects to hijack a special election.
    pub fn hijack_special_election(&mut self, player: usize) -> Result<(), GameError> {
        let GameState::PromptMonarchist { monarchist, hijacked, .. } = &mut self.state else {
            return Err(GameError::InvalidAction);
        };

        if player != *monarchist || !self.players[player].alive {
            return Err(GameError::InvalidAction);
        };

        *hijacked = true;
        Ok(())
    }

    /// Called when the board has finished presenting the special election screen
    pub fn start_special_election(&mut self) -> Result<(), GameError> {
        let GameState::PromptMonarchist { monarchist, last_president, hijacked } = self.state else {
            return Err(GameError::InvalidAction);
        };

        if hijacked {
            self.next_president = Some(NextPresident::Monarchist { monarchist, last_president });
            self.start_round();
        } else {
            self.state = GameState::ChoosePlayer {
                action: ExecutiveAction::SpecialElection,
                can_select: EligiblePlayers::only_one(last_president),
                can_be_selected: self.eligible_players().exclude(last_president).make(),
            };
        }
        Ok(())
    }

    /// Called when the board has finished leaving a "communist session".
    pub fn end_communist_end(&mut self) -> Result<(), GameError> {
        use ExecutiveAction::*;

        let GameState::CommunistEnd { action, chosen_player } = self.state else {
            return Err(GameError::InvalidAction);
        };

        match action {
            Bugging => {
                self.start_round();
            }
            Radicalisation | Congress => {
                if let Some(player_idx) = chosen_player {
                    let player = &mut self.players[player_idx];
                    self.radicalised = player.radicalise();
                }
                self.state = GameState::ActionReveal {
                    action,
                    chosen_player,
                    confirmations: Confirmations::new(self.num_players_alive()),
                };
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    /// Called when the board has finished presenting the executive action.
    pub fn end_executive_action(&mut self, player: Option<usize>) -> Result<(), GameError> {
        use ExecutiveAction::*;

        let GameState::ActionReveal { action, chosen_player, confirmations } = &mut self.state else {
            return Err(GameError::InvalidAction);
        };

        match action {
            // Only the president may end these actions
            InvestigatePlayer | PolicyPeak => {
                let president = self.last_government.unwrap().president;
                if player != Some(president) {
                    return Err(GameError::InvalidAction);
                }
            }
            // Only the board may end these actions
            SpecialElection | Execution | FiveYearPlan | Confession => {
                if player.is_some() {
                    return Err(GameError::InvalidAction);
                }
            }
            // These actions are ended once all players are ready
            Bugging | Radicalisation | Congress => {
                let Some(player) = player else {
                    return Err(GameError::InvalidAction);
                };
                confirmations.confirm(player);
                if !confirmations.can_proceed() {
                    return Ok(());
                }
            }
        };

        match action {
            InvestigatePlayer => {
                self.players[chosen_player.unwrap()].investigated = true;
                self.start_round();
            }
            SpecialElection => {
                let player = chosen_player.unwrap();
                self.next_president = Some(NextPresident::Normal { player });
                self.start_round();
            }
            Execution => {
                let player = &mut self.players[chosen_player.unwrap()];
                player.alive = false;
                player.not_hitler = player.role != Role::Hitler;

                if self.check_game_over() {
                    return Ok(());
                }

                self.start_round();
            }
            Bugging => {
                self.state = GameState::CommunistEnd { action: *action, chosen_player: None };
            }
            FiveYearPlan => {
                self.deck.five_year_plan(&mut self.rng);
                self.start_round();
            }
            _ => {
                self.start_round();
            }
        }
        Ok(())
    }
}
