use super::{player::Role, votes::MonarchistVotes, Game, GameState};
use crate::{
    error::GameError,
    game::{
        confirmations::Confirmations, eligible::EligiblePlayers, government::Government,
        WinCondition,
    },
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
    /// The Anarchist is assassinating a player.
    Assassination,
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
            ExecutiveAction::Assassination => "assassination",
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
        let Government {
            president,
            chancellor,
        } = self.last_government.unwrap();

        match action {
            InvestigatePlayer => {
                self.state = GameState::ChoosePlayer {
                    action,
                    can_select: EligiblePlayers::only_one(president),
                    can_be_selected: self
                        .eligible_players()
                        .not_investigated()
                        .exclude(president)
                        .make(),
                    communist_reveal: false,
                };
            }
            SpecialElection => {
                let monarchist = self.players.iter().position(|p| p.role == Role::Monarchist);
                if let Some(monarchist) = monarchist {
                    self.state = GameState::MonarchistElection {
                        monarchist,
                        president,
                        confirmed: false,
                        monarchist_chancellor: None,
                        president_chancellor: None,
                        eligible_chancellors: self.eligble_chancellors(monarchist),
                        votes: MonarchistVotes::new(self.num_players_alive(), monarchist),
                    };
                } else {
                    self.state = GameState::ChoosePlayer {
                        action,
                        can_select: EligiblePlayers::only_one(president),
                        can_be_selected: self.eligible_players().exclude(president).make(),
                        communist_reveal: false,
                    };
                }
            }
            Execution => {
                self.state = GameState::ChoosePlayer {
                    action,
                    can_select: EligiblePlayers::only_one(president),
                    can_be_selected: self.eligible_players().exclude(president).make(),
                    communist_reveal: false,
                };
            }
            PolicyPeak | FiveYearPlan => {
                self.state = GameState::ActionReveal {
                    action,
                    chosen_player: None,
                    confirmations: Confirmations::new(self.num_players_alive()),
                    communist_reveal: false,
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
                    communist_reveal: false,
                };
            }
            Assassination => panic!("Invalid action"),
        }
    }

    /// Called when the anarchist wishes to execute a player.
    pub fn start_anarchist_assassination(&mut self, player: usize) -> Result<(), GameError> {
        if !matches!(&self.state, GameState::CardReveal { .. }) {
            return Err(GameError::InvalidAction);
        }

        let idx = player;
        let Some(player) = self.players.get(player) else {
            return Err(GameError::InvalidAction);
        };
        if !player.alive || player.role != Role::Anarchist {
            return Err(GameError::InvalidAction);
        }
        if self.assassinated {
            return Err(GameError::InvalidAction);
        }

        self.state = GameState::ChoosePlayer {
            action: ExecutiveAction::Assassination,
            can_select: EligiblePlayers::only_one(idx),
            can_be_selected: self.eligible_players().exclude(idx).make(),
            communist_reveal: true,
        };
        Ok(())
    }

    /// Called when the board has finished entering a "communist session".
    pub fn end_communist_start(&mut self) -> Result<(), GameError> {
        use ExecutiveAction::*;

        let GameState::CommunistStart { action } = self.state else {
            return Err(GameError::InvalidAction);
        };

        if action == Congress && self.radicalised {
            self.state = GameState::Congress;
            return Ok(());
        }

        let can_select = self.eligible_players().ordinary_communist().make();

        let mut can_be_selected = self.eligible_players().not_communist();
        if matches!(action, Radicalisation | Congress) {
            can_be_selected = can_be_selected.not_investigated();
        }
        let can_be_selected = can_be_selected.make();

        self.state = GameState::ChoosePlayer {
            action,
            can_select,
            can_be_selected,
            communist_reveal: false,
        };
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
        let GameState::MonarchistElection { monarchist, confirmed, .. } = &mut self.state else {
            return Err(GameError::InvalidAction);
        };

        if player != *monarchist || !self.players[*monarchist].alive {
            return Err(GameError::InvalidAction);
        };

        *confirmed = true;
        Ok(())
    }

    /// Called when the board has finished giving the monarchist a chance to hijack a special election.
    pub fn start_special_election(&mut self) -> Result<(), GameError> {
        let GameState::MonarchistElection { president, confirmed, .. } = &self.state else {
            return Err(GameError::InvalidAction);
        };

        if *confirmed {
            return Err(GameError::InvalidAction);
        }

        self.state = GameState::ChoosePlayer {
            action: ExecutiveAction::SpecialElection,
            can_select: EligiblePlayers::only_one(*president),
            can_be_selected: self.eligible_players().exclude(*president).make(),
            communist_reveal: false,
        };
        Ok(())
    }

    /// Called when the board has finished leaving a "communist session".
    pub fn end_communist_end(&mut self) -> Result<(), GameError> {
        let GameState::CommunistEnd { action, chosen_player } = self.state else {
            return Err(GameError::InvalidAction);
        };

        self.state = GameState::ActionReveal {
            action,
            chosen_player,
            confirmations: Confirmations::new(self.num_players_alive()),
            communist_reveal: false,
        };
        Ok(())
    }

    /// Called when the board has finished presenting the executive action.
    pub fn end_executive_action(&mut self, player: Option<usize>) -> Result<(), GameError> {
        use ExecutiveAction::*;

        let GameState::ActionReveal { action, chosen_player, confirmations, communist_reveal } = &mut self.state else {
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
            SpecialElection | Execution | FiveYearPlan | Confession | Assassination => {
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
                self.start_election(None);
            }
            SpecialElection => {
                let player = *chosen_player;
                self.start_election(player);
            }
            Execution | Assassination => {
                let player = &mut self.players[chosen_player.unwrap()];
                player.alive = false;
                player.not_hitler = player.role != Role::Hitler;

                if *communist_reveal && player.role == Role::Capitalist {
                    self.state = GameState::GameOver(WinCondition::CapitalistExecuted);
                    return Ok(());
                }
                if self.check_game_over() {
                    return Ok(());
                }

                self.start_election(None);
            }
            Radicalisation | Congress => {
                if let Some(player) = chosen_player {
                    let player = &mut self.players[*player];
                    if matches!(player.role, Role::Liberal | Role::Centrist) {
                        player.role = Role::Communist;
                        self.radicalised = true;
                    }
                }

                self.start_election(None);
            }
            _ => {
                self.start_election(None);
            }
        }
        Ok(())
    }
}
