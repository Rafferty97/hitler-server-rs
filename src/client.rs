use crate::{
    error::GameError,
    game::{Game as GameInner, GameOptions},
    session::{GameUpdate, SessionHandle, SessionManager},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::watch;

/// A single game client, which could for a board or a player.
pub struct Client<'a> {
    manager: &'a SessionManager,
    session: Option<SessionHandle>,
    player: Option<String>,
    game_id: Option<String>,
    updates: Option<watch::Receiver<GameUpdate>>,
}

/// An action performed by the board.
#[derive(Serialize, Deserialize)]
pub enum BoardAction {
    EndVoting,
    // FIXME
}

/// An action performed by the player.
#[derive(Serialize, Deserialize)]
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
    // FIXME
}

impl<'a> Client<'a> {
    /// Creates a new game client.
    pub fn new(manager: &'a SessionManager) -> Self {
        Self {
            manager,
            session: None,
            game_id: None,
            player: None,
            updates: None,
        }
    }

    /// Creates a new game session, returning its ID.
    pub fn create_game(&mut self, options: GameOptions) -> Result<String, GameError> {
        let session = self.manager.create_game(options);
        let id = session.lock().unwrap().id().to_owned();
        Ok(id)
    }

    /// Joins a game as a board.
    pub fn join_as_board(&mut self, game_id: &str) -> Result<(), GameError> {
        let session = self.manager.find_game(game_id)?;
        self.player = None;
        self.game_id = Some(game_id.to_string());
        self.updates = Some(session.lock().unwrap().subscribe());
        self.session = Some(session);
        Ok(())
    }

    /// Joins a game as a player.
    pub fn join_as_player(&mut self, game_id: &str, name: &str) -> Result<(), GameError> {
        let session = self.manager.find_game(game_id)?;
        {
            let mut session = session.lock().unwrap();
            session.add_player(name)?;
            self.player = Some(name.to_string());
            self.game_id = Some(game_id.to_string());
            self.updates = Some(session.subscribe());
        }
        self.session = Some(session);
        Ok(())
    }

    /// Waits until there is an update to the game state, then returns the latest state.
    pub async fn next_state(&mut self) -> Value {
        if let Some(updates) = &mut self.updates {
            updates.changed().await.ok();
            let update = updates.borrow();
            if let Some(name) = &self.player {
                json!({
                    "game_id": self.game_id,
                    "players": update.players,
                    "mode": "player",
                    "state": update.player_updates.iter().find(|u| &u.name == name),
                })
            } else {
                json!({
                    "game_id": self.game_id,
                    "players": update.players,
                    "mode": "board",
                    "state": update.board_update,
                })
            }
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

    /// Called when the board performs an action.
    pub fn board_action(&self, action: BoardAction) -> Result<(), GameError> {
        if self.player.is_some() {
            return Err(GameError::InvalidAction);
        }
        self.mutate_game(|game| match action {
            BoardAction::EndVoting => game.end_voting(),
            // "cardReveal" => game.end_card_reveal(None),
            // "executiveAction" => game.end_executive_action(None),
            // "legislativeSession" => game.end_legislative_session(),
            // FIXME
            _ => Err(GameError::InvalidAction),
        })
    }

    /// Called when a player performs an action.
    pub fn player_action(&self, action: PlayerAction) -> Result<(), GameError> {
        let player = self.player.as_ref().ok_or(GameError::InvalidAction)?;
        self.mutate_game(|game| {
            let player = game.find_player(&player)?;
            match &action {
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
                // FIXME
            }
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
