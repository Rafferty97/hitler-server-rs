use super::{player::Role, votes::MonarchistVotes, Game, GameState};
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
    fn start_executive_action(&mut self, action: ExecutiveAction) {
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
                };
            }
            SpecialElection | Execution => {
                self.state = GameState::ChoosePlayer {
                    action,
                    can_select: EligiblePlayers::only_one(president),
                    can_be_selected: self.eligible_players().exclude(president).make(),
                };
            }
            PolicyPeak => self.reveal_executive_action(action),
            Bugging | Radicalisation | Congress => {
                self.state = GameState::CommunistStart { action };
            }
            FiveYearPlan => self.reveal_executive_action(action),
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
        };
        Ok(())
    }

    /// Called when a player is chosen as the subject of an executive action.
    pub fn choose_player_for_action(&mut self, action: ExecutiveAction, player: usize) {
        use ExecutiveAction::*;
        match action {
            InvestigatePlayer | SpecialElection | Execution | Confession => {
                self.state = GameState::ActionReveal {
                    action,
                    chosen_player: Some(player),
                    confirmations: Confirmations::new(self.num_players_alive()),
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
    }

    /// Called when the monarchist elects to hijack a special election.
    pub fn hijack_special_election(&mut self, player: usize) -> Result<(), GameError> {
        let (action, president_chancellor) = match &self.state {
            GameState::ChoosePlayer { action, .. } => (action, None),
            GameState::ActionReveal {
                action,
                chosen_player,
                ..
            } => (action, chosen_player),
            _ => return Err(GameError::InvalidAction),
        };
        if action != ExecutiveAction::SpecialElection {
            return Err(GameError::InvalidAction);
        }

        let monarchist = player;
        let Some(player) = self.players.get(player) else {
            return Err(GameError::InvalidAction);
        };
        if !player.alive || player.role != Role::Monarchist {
            return Err(GameError::InvalidAction);
        }

        self.state = GameState::MonarchistElection {
            monarchist,
            president: self.last_government.unwrap().president,
            monarchist_chancellor: None,
            president_chancellor,
            eligible_chancellors: self.eligble_chancellors(monarchist),
            votes: MonarchistVotes::new(self.num_players_alive(), monarchist),
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
        };
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
                if player != None {
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
                self.start_election(chosen_player);
            }
            Execution => {
                let player = &mut self.players[chosen_player.unwrap()];
                player.alive = false;
                player.not_hitler = player.role != Role::Hitler;
                if self.check_game_over() {
                    return Ok(());
                }
                self.start_election(None);
            }
            Radicalisation | Congress => {
                if let Some(player) = chosen_player {
                    let player = &mut self.players[player];
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
