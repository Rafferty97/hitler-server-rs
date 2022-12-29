use crate::game::{BoardUpdate, GameOptions, PlayerUpdate, PublicPlayer};
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
    /// Channel for sending game state updates.
    updates: watch::Sender<GameUpdate>,
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
    GameOver,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct GameUpdate {
    pub players: Vec<PublicPlayer>,
    pub board_update: Option<BoardUpdate>,
    pub player_updates: Vec<PlayerUpdate>,
    pub ended: bool,
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
            updates: watch::channel(GameUpdate::default()).0,
            dbs,
            last_ts: Instant::now(),
        }
    }

    /// Gets the unique game ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Adds the player to the game if there are not already a member,
    /// unless the game is unable to accept any new players.
    pub fn add_player(&mut self, name: &str) -> Result<(), GameError> {
        match &mut self.game {
            Game::Lobby { options, players } => {
                if players.iter().find(|n| *n == name).is_some() {
                    return Ok(());
                }
                if players.len() == options.max_players() {
                    return Err(GameError::TooManyPlayers);
                }
                players.push(name.to_string());
                Ok(())
            }
            Game::Playing { game, .. } => match game.find_player(name) {
                Ok(_) => Ok(()),
                Err(_) => Err(GameError::CannotJoinStartedGame),
            },
            Game::GameOver => Err(GameError::GameNotFound),
        }
    }

    /// Called by a new client to subscribe to game state updates.
    pub fn subscribe(&mut self) -> watch::Receiver<GameUpdate> {
        let rx = self.updates.subscribe();
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
        self.game = Game::GameOver;
        self.notify();
        self.persist_game().ok();

        Ok(())
    }

    /// Notifies all connected clients of the new game state.
    fn notify(&mut self) {
        match &self.game {
            Game::Lobby { players, .. } => self.updates.send_replace(Self::lobby_update(players)),
            Game::Playing { game, .. } => self.updates.send_replace(Self::game_update(game)),
            Game::GameOver => self.updates.send_replace(Self::game_over_update()),
        };
        self.last_ts = Instant::now();
    }

    /// Creates a lobby game update.
    fn lobby_update(players: &[String]) -> GameUpdate {
        let make_player = |name: &String| PublicPlayer {
            name: name.clone(),
            alive: true,
            not_hitler: false,
        };
        GameUpdate {
            players: players.iter().map(make_player).collect(),
            board_update: None,
            player_updates: vec![],
            ended: false,
        }
    }

    /// Create a game update.
    fn game_update(game: &GameInner) -> GameUpdate {
        GameUpdate {
            players: game.get_public_players(),
            board_update: Some(game.get_board_update()),
            player_updates: (0..game.num_players())
                .map(|i| game.get_player_update(i))
                .collect(),
            ended: false,
        }
    }

    /// Creates a game over update.
    fn game_over_update() -> GameUpdate {
        GameUpdate {
            players: vec![],
            board_update: None,
            player_updates: vec![],
            ended: true,
        }
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
                "outcome": game.outcome()
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
            Game::GameOver => 0,
        }
    }

    fn player_names(&self) -> Vec<String> {
        match self {
            Game::Lobby { players, .. } => players.clone(),
            Game::Playing { game, .. } => game.player_names().map(|s| s.to_string()).collect(),
            Game::GameOver => vec![],
        }
    }

    fn game_mut(&mut self) -> Option<&mut GameInner> {
        match self {
            Game::Lobby { .. } => None,
            Game::Playing { game, .. } => Some(game),
            Game::GameOver => None,
        }
    }

    fn can_start(&self) -> bool {
        match self {
            Game::Lobby { .. } => true,
            Game::Playing { game, .. } => game.game_over(),
            Game::GameOver => false,
        }
    }

    fn can_end(&self) -> bool {
        match self {
            Game::Lobby { .. } => false,
            Game::Playing { game, .. } => game.game_over(),
            Game::GameOver => false,
        }
    }
}
