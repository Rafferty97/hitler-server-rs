use crate::{error::GameError, game::Game};
use dashmap::{mapref::entry::Entry, DashMap};
use rand::{Rng, RngCore};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

#[derive(Default)]
pub struct SessionManager {
    sessions: DashMap<String, SessionHandle>,
}

pub type SessionHandle = Arc<Mutex<Session>>;

impl SessionManager {
    pub fn create_game(&self) -> SessionHandle {
        loop {
            let id = Self::random_id();
            let entry = self.sessions.entry(id);
            if let Entry::Occupied(_) = entry {
                continue;
            }
            let session = Session::new(entry.key().clone());
            let session = Arc::new(Mutex::new(session));
            entry.or_insert(session.clone());
            break session;
        }
    }

    pub fn find_game(&self, game_id: &str) -> Result<SessionHandle, GameError> {
        self.sessions
            .get(game_id)
            .map(|session| session.clone())
            .ok_or(GameError::GameNotFound)
    }

    fn random_id() -> String {
        let mut rng = rand::thread_rng();
        (0..4).map(|_| rng.gen_range('A'..='Z')).collect()
    }
}

pub struct Session {
    /// The game ID.
    id: String,
    /// The players in the game.
    players: Vec<Player>,
    /// The game itself once it has started.
    game: Option<Game>,
    /// Channel for sending board state updates.
    board_state: watch::Sender<Value>,
}

struct Player {
    /// The player name.
    name: String,
    /// Channel for sending player state updates.
    player_state: watch::Sender<Value>,
}

/// Represents an active Secret Hitler game.
impl Session {
    pub fn new(id: String) -> Self {
        Self {
            id,
            players: vec![],
            game: None,
            board_state: watch::channel(Value::Null).0,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn get_player(&mut self, name: &str) -> Result<usize, GameError> {
        if let Some(idx) = self.players.iter().position(|player| player.name == name) {
            return Ok(idx);
        }

        if self.players.len() == 10 {
            return Err(GameError::TooManyPlayers);
        }

        self.players.push(Player {
            name: name.to_string(),
            player_state: watch::channel(Value::Null).0,
        });
        Ok(self.players.len() - 1)
    }

    pub fn join_board(&self) -> watch::Receiver<Value> {
        let rx = self.board_state.subscribe();
        self.notify();
        rx
    }

    pub fn join_player(&mut self, player: usize) -> watch::Receiver<Value> {
        let rx = self.players[player].player_state.subscribe();
        self.notify();
        rx
    }

    fn notify(&self) {
        if let Some(game) = self.game.as_ref() {
            // A game is in session
            self.board_state.send_replace(game.get_board_json());
            for (idx, player) in self.players.iter().enumerate() {
                let state = game.get_player_json(idx);
                player.player_state.send_replace(state);
            }
        } else {
            // The game is still in "lobby mode"
            let names = self.player_names();
            self.board_state
                .send_replace(Game::get_lobby_board_json(&names));
            for (idx, player) in self.players.iter().enumerate() {
                let state = Game::get_lobby_player_json(&names, idx);
                player.player_state.send_replace(state);
            }
        }
    }

    fn player_names(&self) -> Vec<String> {
        self.players
            .iter()
            .map(|p| p.name.to_string())
            .collect::<Vec<_>>()
    }
}

/// A single game client, which could be a board or a player.
pub struct Client<'a> {
    manager: &'a SessionManager,
    session: Option<Arc<Mutex<Session>>>,
    player: Option<usize>,
    state: Option<watch::Receiver<Value>>,
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
            let player = session.get_player(name)?;
            self.state = Some(session.join_player(player));
            self.player = Some(player);
        }
        self.session = Some(session);
        Ok(())
    }

    /// Gets the current state of the game, from the board or player's perspective.
    pub fn state(&self) -> Value {
        if let Some(state) = &self.state {
            state.borrow().clone()
        } else {
            Value::Null
        }
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

        let names = session
            .players
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>();
        let seed = rand::thread_rng().next_u64();
        session.game = Some(Game::new(&names, seed));
        session.notify();

        Ok(())
    }

    /// Called when the board is ready to move on.
    pub fn board_next(&self) -> Result<(), GameError> {
        if self.player.is_some() {
            return Err(GameError::InvalidAction);
        }
        self.mutate_game(|game| game.board_next())
    }

    /// Called when a player is ready to move on.
    pub fn player_next(&self) -> Result<(), GameError> {
        let Some(player) = self.player else {
            return Err(GameError::InvalidAction);
        };
        self.mutate_game(|game| game.player_next(player))
    }

    fn mutate_game(
        &self,
        mut f: impl FnMut(&mut Game) -> Result<(), GameError>,
    ) -> Result<(), GameError> {
        let Some(session) = &self.session else {
            return Err(GameError::InvalidAction);
        };
        let mut session = session.lock().unwrap();
        let Some(game) = &mut session.game else {
            return Err(GameError::InvalidAction);
        };

        f(game)?;
        session.notify();

        Ok(())
    }
}
