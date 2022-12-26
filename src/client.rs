use crate::{
    error::GameError,
    game::Game as GameInner,
    session::{SessionHandle, SessionManager},
};
use serde_json::Value;
use tokio::sync::watch;

/// A single game client, which could be a board or a player.
pub struct Client<'a> {
    manager: &'a SessionManager,
    session: Option<SessionHandle>,
    player: Option<usize>,
    state: Option<watch::Receiver<Value>>,
}

/// An action performed by the player.
pub enum PlayerAction {
    EndNightRound,
    EndCardReveal,
    EndExecutiveAction,
    ChoosePlayer { name: String },
    CastVote { vote: bool },
    Discard { index: usize },
    VetoAgenda,
    AcceptVeto,
    RejectVeto,
}

impl<'a> Client<'a> {
    /// Creates a new game client.
    pub fn new(manager: &'a SessionManager) -> Self {
        Self {
            manager,
            session: None,
            player: None,
            state: None,
        }
    }

    /// Creates a new game session, returning its ID.
    pub fn create_game(&mut self) -> Result<String, GameError> {
        let session = self.manager.create_game();
        let id = session.lock().unwrap().id().to_owned();
        Ok(id)
    }

    /// Joins a game as a board.
    pub fn join_as_board(&mut self, game_id: &str) -> Result<(), GameError> {
        let session = self.manager.find_game(game_id)?;
        self.state = Some(session.lock().unwrap().join_board());
        self.session = Some(session);
        self.player = None;
        Ok(())
    }

    /// Joins a game as a player.
    pub fn join_as_player(&mut self, game_id: &str, name: &str) -> Result<(), GameError> {
        let session = self.manager.find_game(game_id)?;
        {
            let mut session = session.lock().unwrap();
            let player = session.get_or_insert_player(name)?;
            self.state = Some(session.join_player(player));
            self.player = Some(player);
        }
        self.session = Some(session);
        Ok(())
    }

    /// Waits until there is an update to the game state, then returns the latest state.
    pub async fn next_state(&mut self) -> Value {
        if let Some(state) = &mut self.state {
            state.changed().await.ok();
            state.borrow().clone()
        } else {
            std::future::pending().await
        }
    }

    /// Starts a new game of Secret Hitler.
    pub fn start_game(&self) -> Result<(), GameError> {
        let Some(session) = &self.session else {
            return Err(GameError::InvalidAction);
        };
        let mut session = session.lock().unwrap();
        session.start_game()
    }

    /// Called when the board is ready to move on.
    pub fn board_next(&self, state: &str) -> Result<(), GameError> {
        if self.player.is_some() {
            return Err(GameError::InvalidAction);
        }
        self.mutate_game(|game| match state {
            "election" => game.end_voting(),
            "cardReveal" => game.end_card_reveal(None),
            "executiveAction" => game.end_executive_action(None),
            "legislativeSession" => game.end_legislative_session(),
            _ => Err(GameError::InvalidAction),
        })
    }

    /// Called when a player performs an action.
    pub fn player_action(&self, action: PlayerAction) -> Result<(), GameError> {
        let player = self.player.ok_or(GameError::InvalidAction)?;
        self.mutate_game(|game| match &action {
            PlayerAction::EndNightRound => game.end_night_round(player),
            PlayerAction::EndCardReveal => game.end_card_reveal(Some(player)),
            PlayerAction::EndExecutiveAction => game.end_executive_action(Some(player)),
            PlayerAction::CastVote { vote } => game.cast_vote(player, *vote),
            PlayerAction::ChoosePlayer { name } => {
                let other = game.find_player(name)?;
                game.choose_player(player, other)
            }
            PlayerAction::Discard { index } => game.discard_policy(player, *index),
            PlayerAction::VetoAgenda => game.veto_agenda(player),
            PlayerAction::AcceptVeto => game.veto_agenda(player),
            PlayerAction::RejectVeto => game.reject_veto(player),
        })
    }

    /// Keeps the game session alive.
    pub fn heartbeat(&self) {
        let Some(session) = &self.session else {
            return;
        };
        let mut session = session.lock().unwrap();
        session.heartbeat();
    }

    /// Ends the game.
    pub fn end_game(&self) -> Result<(), GameError> {
        let Some(session) = &self.session else {
            return Err(GameError::InvalidAction);
        };
        let mut session = session.lock().unwrap();
        session.end_game()
    }

    /// Performs an action on the game.
    fn mutate_game<F>(&self, mutation: F) -> Result<(), GameError>
    where
        F: FnOnce(&mut GameInner) -> Result<(), GameError>,
    {
        let Some(session) = &self.session else {
            return Err(GameError::InvalidAction);
        };
        let mut session = session.lock().unwrap();
        session.mutate_game(mutation)
    }
}
