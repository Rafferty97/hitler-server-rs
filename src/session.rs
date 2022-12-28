use crate::game::GameOptions;
use crate::time::iso8601;
use crate::{error::GameError, game::Game as GameInner};
use dashmap::{mapref::entry::Entry, DashMap};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::watch;

/// Manages all the game sessions running on the server.
pub struct SessionManager {
    sessions: DashMap<String, SessionHandle>,
    dbs: Dbs,
}

/// The databases that games are persisted to.
#[derive(Clone)]
struct Dbs {
    db: sled::Db,
    game: sled::Tree,
    archive: sled::Tree,
}

/// A single game session.
pub struct Session {
    /// The game ID.
    id: String,
    /// The game itself.
    game: Game,
    /// Channel for sending game state updates to boards.
    board_state: watch::Sender<Value>,
    /// Channels for sending game state updates to players.
    player_states: Vec<watch::Sender<Value>>,
    /// The databases.
    dbs: Dbs,
    /// Timestamp of the last time this session was interacted with.
    last_ts: Instant,
}

pub type SessionHandle = Arc<Mutex<Session>>;

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize)]
enum Game {
    Lobby {
        options: GameOptions,
        players: Vec<String>,
    },
    Playing {
        /// The game itself.
        game: GameInner,
        /// Timestamp that the game was created.
        started_ts: std::time::SystemTime,
        /// Whether this game has been archived.
        archived: bool,
    },
    Over,
}

impl SessionManager {
    pub fn new(db: sled::Db) -> Result<Self, Box<dyn Error>> {
        let sessions = DashMap::new();
        let dbs = Dbs {
            db: db.clone(),
            game: db.open_tree("games")?,
            archive: db.open_tree("archive")?,
        };
        for entry in dbs.game.iter() {
            let (id, game) = entry?;
            let id = String::from_utf8(id.to_vec())?;
            let Ok(game) = serde_json::from_slice(&game) else {
                continue;
            };
            let session = Session::hydrate(id.clone(), dbs.clone(), game);
            let session = Arc::new(Mutex::new(session));
            sessions.insert(id, session);
        }
        Ok(Self { sessions, dbs })
    }

    pub fn create_game(&self, options: GameOptions) -> SessionHandle {
        loop {
            let id = Self::random_id();
            let entry = self.sessions.entry(id);
            if let Entry::Occupied(_) = entry {
                continue;
            }
            let session = Session::new(entry.key().clone(), self.dbs.clone(), options);
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

    pub fn num_games(&self) -> usize {
        self.sessions.len()
    }

    pub fn purge_games(&self) {
        let mut ids_to_delete = vec![];

        // Find expired sessions
        for session in self.sessions.iter() {
            let game_id = session.key();
            let Ok(session) = session.lock() else {
                log::error!("Found poisoned session: {}", game_id);
                ids_to_delete.push(session.key().clone());
                continue;
            };
            let elapsed = Instant::now().duration_since(session.last_ts);
            if elapsed > Duration::from_secs(3600) {
                if self.dbs.game.remove(session.id().as_bytes()).is_ok() {
                    ids_to_delete.push(game_id.clone());
                } else {
                    log::error!("Could not remove game: {}", game_id);
                }
            }
        }

        // Delete the archived games
        for game_id in ids_to_delete.into_iter() {
            self.sessions.remove(&game_id);
        }
    }

    fn random_id() -> String {
        let mut rng = rand::thread_rng();
        (0..4)
            .map(|_| match rng.gen_range('A'..='Z') {
                // Avoid U and V because the "hitler font" can't distinguish them
                'U' => 'A',
                'V' => 'B',
                other => other,
            })
            .collect()
    }
}

impl Session {
    fn new(id: String, dbs: Dbs, options: GameOptions) -> Self {
        let game = Game::Lobby {
            options,
            players: vec![],
        };
        Self::hydrate(id, dbs, game)
    }

    fn hydrate(id: String, dbs: Dbs, game: Game) -> Self {
        let mut player_states = vec![];
        for _ in 0..game.num_players() {
            player_states.push(watch::channel(Value::Null).0);
        }
        Self {
            id,
            game,
            board_state: watch::channel(Value::Null).0,
            player_states,
            dbs,
            last_ts: Instant::now(),
        }
    }

    /// Gets the unique game ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Gets the index of the player with the given name,
    /// adding the player to the game if no player with that name has joined yet.
    pub fn get_or_insert_player(&mut self, name: &str) -> Result<usize, GameError> {
        match &mut self.game {
            Game::Lobby { options, players } => {
                if let Some(idx) = players.iter().position(|n| n == name) {
                    return Ok(idx);
                }
                if players.len() == options.max_players() {
                    return Err(GameError::TooManyPlayers);
                }
                self.player_states.push(watch::channel(Value::Null).0);
                players.push(name.to_string());
                Ok(players.len() - 1)
            }
            Game::Playing { game, .. } => game.find_player(name),
            Game::Over => Err(GameError::GameNotFound),
        }
    }

    /// Called by a new game board client, and returns a stream of updates for the game board.
    pub fn join_board(&mut self) -> watch::Receiver<Value> {
        let rx = self.board_state.subscribe();
        self.notify();
        rx
    }

    /// Called by a new player client, and returns a stream of updates for that player.
    pub fn join_player(&mut self, player: usize) -> watch::Receiver<Value> {
        let rx = self.player_states[player].subscribe();
        self.notify();
        rx
    }

    /// Starts the game.
    pub fn start_game(&mut self) -> Result<(), GameError> {
        // Check there isn't already a game in progress
        if !self.game.can_start() {
            return Err(GameError::InvalidAction);
        }

        // FIXME
        let opts = GameOptions::default();

        self.archive().ok();
        let names = self.game.player_names();
        let seed = rand::thread_rng().next_u64();
        self.game = Game::Playing {
            game: GameInner::new(opts, &names, seed)?,
            started_ts: SystemTime::now(),
            archived: false,
        };
        self.notify();
        self.persist_game().ok();

        Ok(())
    }

    /// Performs an action on the game.
    pub fn mutate_game<F>(&mut self, mutation: F) -> Result<(), GameError>
    where
        F: FnOnce(&mut GameInner) -> Result<(), GameError>,
    {
        let Some(game) = self.game.game_mut() else {
            return Err(GameError::InvalidAction);
        };

        mutation(game)?;
        self.notify();
        self.persist_game().ok();
        self.archive().ok();

        Ok(())
    }

    /// Keeps the game session alive.
    pub fn heartbeat(&mut self) {
        self.last_ts = Instant::now();
    }

    /// Ends the game.
    pub fn end_game(&mut self) -> Result<(), GameError> {
        // Check the game is over.
        if !self.game.can_end() {
            return Err(GameError::InvalidAction);
        }

        self.archive().ok();
        self.game = Game::Over;
        self.notify();
        self.persist_game().ok();

        Ok(())
    }

    /// Notifies all connected clients of the new game state.
    fn notify(&mut self) {
        match &self.game {
            Game::Lobby { players, .. } => {
                let state = GameInner::get_lobby_board_json(players);
                self.board_state.send_replace(state);
                for (idx, player_state) in self.player_states.iter().enumerate() {
                    let state = GameInner::get_lobby_player_json(players, idx);
                    player_state.send_replace(state);
                }
            }
            Game::Playing { game, .. } => {
                self.board_state.send_replace(game.get_board_json());
                for (idx, player_state) in self.player_states.iter().enumerate() {
                    let state = game.get_player_json(idx);
                    player_state.send_replace(state);
                }
            }
            Game::Over => {
                self.board_state.send_replace(json!({ "type": "gameover" }));
                for player_state in self.player_states.iter() {
                    player_state.send_replace(json!({ "type": "gameover" }));
                }
            }
        }
        self.last_ts = Instant::now();
    }

    /// Persists the game state to disk, so it can be recovered upon server restart.
    fn persist_game(&mut self) -> Result<(), Box<dyn Error>> {
        self.dbs.game.insert(
            self.id.as_bytes(),
            serde_json::to_string(&self.game)?.as_bytes(),
        )?;
        Ok(())
    }

    /// Archives the game if it is over and hasn't been archived yet.
    fn archive(&mut self) -> Result<(), Box<dyn Error>> {
        let Game::Playing { game, started_ts, archived } = &mut self.game else {
            return Ok(());
        };
        if game.game_over() && !*archived {
            let key = self.dbs.db.generate_id()?.to_be_bytes();
            let data = json!({
                "game_id": self.id,
                "players": game.player_names().collect::<Value>(),
                "started": iso8601(*started_ts),
                "finished": iso8601(SystemTime::now()),
                "outcome": game.get_outcome_json()
            })
            .to_string();
            self.dbs.archive.insert(key, data.as_bytes())?;
            *archived = true;
        }
        Ok(())
    }
}

impl Game {
    fn num_players(&self) -> usize {
        match self {
            Game::Lobby { players, .. } => players.len(),
            Game::Playing { game, .. } => game.num_players(),
            Game::Over => 0,
        }
    }

    fn player_names(&self) -> Vec<String> {
        match self {
            Game::Lobby { players, .. } => players.clone(),
            Game::Playing { game, .. } => game.player_names().map(|s| s.to_string()).collect(),
            Game::Over => vec![],
        }
    }

    fn game_mut(&mut self) -> Option<&mut GameInner> {
        match self {
            Game::Lobby { .. } => None,
            Game::Playing { game, .. } => Some(game),
            Game::Over => None,
        }
    }

    fn can_start(&self) -> bool {
        match self {
            Game::Lobby { .. } => true,
            Game::Playing { game, .. } => game.game_over(),
            Game::Over => false,
        }
    }

    fn can_end(&self) -> bool {
        match self {
            Game::Lobby { .. } => false,
            Game::Playing { game, .. } => game.game_over(),
            Game::Over => false,
        }
    }
}
