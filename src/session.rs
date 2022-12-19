use crate::game::{Game, GameError};
use chashmap::CHashMap;
use rand::RngCore;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

#[derive(Default)]
pub struct SessionManager {
    sessions: CHashMap<String, SessionHandle>,
}

pub type SessionHandle = Arc<Mutex<Session>>;

impl SessionManager {
    pub fn create_game(&self) -> SessionHandle {
        // FIXME
        let id = "AAAA".to_string();
        let session = Session::new(id.clone());
        let session = Arc::new(Mutex::new(session));
        self.sessions.insert(id, session.clone());
        session
    }

    pub fn find_game(&self, id: &str) -> Option<SessionHandle> {
        self.sessions.get(id).map(|session| session.clone())
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

impl Session {
    pub fn new(id: String) -> Self {
        Self {
            id,
            players: vec![],
            game: None,
            board_state: watch::channel(Self::get_lobby_json(&[])).0,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn join_board(&self) -> watch::Receiver<Value> {
        let rx = self.board_state.subscribe();
        self.notify();
        rx
    }

    pub fn join_player(&mut self, name: String) -> Result<watch::Receiver<Value>, GameError> {
        // Find an existing player, or create a new one
        let player = self.players.iter().find(|player| player.name == name);
        let player = match player {
            Some(player) => player,
            None => {
                if self.players.len() == 10 {
                    return Err(GameError::TooManyPlayers);
                }
                self.players.push(Player {
                    name: name.clone(),
                    player_state: watch::channel(self.get_player_json(&name)).0,
                });
                self.players.last().unwrap()
            }
        };
        let rx = player.player_state.subscribe();
        self.notify();
        Ok(rx)
    }

    fn notify(&self) {
        self.board_state.send(self.get_board_json()).ok();
        for player in &self.players {
            player
                .player_state
                .send(self.get_player_json(&player.name))
                .ok();
        }
    }

    fn get_board_json(&self) -> Value {
        self.game
            .as_ref()
            .map(|game| game.get_board_json())
            .unwrap_or_else(|| Self::get_lobby_json(&self.players))
    }

    fn get_player_json(&self, name: &str) -> Value {
        self.game
            .as_ref()
            .map(|game| game.get_player_json(name).unwrap())
            .unwrap_or_else(|| {
                json!({
                    "id": name,
                    "name": name,
                    "action": {
                        "type": "lobby",
                        "canStart": self.players.len() >= 5
                    },
                    "players": Self::get_players_json(&self.players),
                    "isDead": false
                })
            })
    }

    fn get_lobby_json(players: &[Player]) -> Value {
        json!({
            "players": Self::get_players_json(players),
            "state": { "type": "lobby" },
            "electionTracker": 0,
            "numLiberalCards": 0,
            "numFascistCards": 0,
            "drawPile": 0,
            "lastPresident": -1,
            "lastChancellor": -1,
        })
    }

    fn get_players_json(players: &[Player]) -> Value {
        players
            .iter()
            .map(|player| {
                json!({
                    "id": player.name,
                    "name": player.name,
                    "isDead": false,
                    "isConfirmedNotHitler": false,
                    "hasBeenInvestigated": false
                })
            })
            .collect::<Value>()
    }
}

pub struct Client {
    session: Option<Arc<Mutex<Session>>>,
    name: Option<String>,
    state: Option<watch::Receiver<Value>>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            session: None,
            name: None,
            state: None,
        }
    }

    pub fn join_as_board(&mut self, session: SessionHandle) {
        self.state = Some(session.lock().unwrap().join_board());
        self.session = Some(session);
        self.name = None;
    }

    pub fn join_as_player(&mut self, session: SessionHandle, name: &str) -> Result<(), GameError> {
        self.state = Some(session.lock().unwrap().join_player(name.to_string())?);
        self.session = Some(session);
        self.name = Some(name.to_string());
        Ok(())
    }

    pub async fn next_state(&mut self) -> Value {
        if let Some(state) = &mut self.state {
            state.changed().await.ok(); // FIXME: Errors?
            state.borrow().clone()
        } else {
            std::future::pending().await
        }
    }

    pub fn board_next(&self) -> Result<(), GameError> {
        if self.name.is_some() {
            return Err(GameError::InvalidAction);
        }
        let Some(session) = &self.session else {
            return Err(GameError::InvalidAction);
        };
        let mut session = session.lock().unwrap();
        let Some(game) = &mut session.game else {
            return Err(GameError::InvalidAction);
        };
        let result = game.board_next();
        if result.is_ok() {
            session.notify();
        }
        result
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
}
