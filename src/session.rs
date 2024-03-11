use crate::game::{BoardUpdate, GameOptions, PlayerUpdate, PublicPlayer, WinCondition};
use crate::{error::GameError, game::Game as GameInner};
use chrono::{DateTime, Utc};
use dashmap::{mapref::entry::Entry, DashMap};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sled::CompareAndSwapError;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::watch;

/// Manages all the game sessions running on the server.
pub struct SessionManager {
    sessions: DashMap<String, SessionHandle>,
    db: Database,
}

/// The databases that games are persisted to.
#[derive(Clone)]
struct Database {
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
    db: Database,
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
        min_players: usize,
        max_players: usize,
    },
    Playing {
        /// The game itself.
        game: GameInner,
        /// Timestamp that the game was created.
        started_ts: DateTime<Utc>,
        /// Whether this game has been archived.
        archived: bool,
    },
    #[allow(clippy::enum_variant_names)]
    GameOver,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct GameUpdate {
    pub lifecycle: GameLifecycle,
    pub players: Vec<PublicPlayer>,
    pub board_update: Option<BoardUpdate>,
    pub player_updates: Vec<PlayerUpdate>,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum GameLifecycle {
    Lobby { can_start: bool },
    Playing,
    Ended,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameStats {
    pub id: String,
    pub players: Vec<String>,
    pub started: DateTime<Utc>,
    pub finished: DateTime<Utc>,
    pub outcome: WinCondition,
}

impl SessionManager {
    pub fn new(db: sled::Db) -> Result<Self, Box<dyn Error>> {
        let sessions = DashMap::new();
        let db = Database {
            game: db.open_tree("games")?,
            archive: db.open_tree("archive")?,
        };
        for entry in db.game.iter() {
            let (id, game) = entry?;
            let id = String::from_utf8(id.to_vec())?;
            let Ok(game) = serde_json::from_slice(&game) else {
                continue;
            };
            let session = Session::hydrate(id.clone(), db.clone(), game);
            let session = Arc::new(Mutex::new(session));
            sessions.insert(id, session);
        }
        Ok(Self { sessions, db })
    }

    pub fn create_game(&self, options: GameOptions) -> Result<SessionHandle, GameError> {
        loop {
            let id = Self::random_id();
            let entry = self.sessions.entry(id);
            if let Entry::Occupied(_) = entry {
                continue;
            }
            let session = Session::new(entry.key().clone(), self.db.clone(), options)?;
            let session = Arc::new(Mutex::new(session));
            entry.or_insert(session.clone());
            break Ok(session);
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
        let max_idle = Duration::from_secs(3600);
        let mut ids_to_delete = vec![];

        // Find expired sessions and delete them from sled
        for session in self.sessions.iter() {
            let game_id = session.key();
            let expired = session.lock().map_or(true, |s| s.last_ts.elapsed() > max_idle);
            if expired {
                match self.db.game.remove(game_id) {
                    Ok(_) => ids_to_delete.push(game_id.clone()),
                    Err(err) => log::error!("Could not remove game: {}: {}", game_id, err),
                }
            }
        }

        // Remove the deleted sessions from cache
        for game_id in ids_to_delete.into_iter() {
            self.sessions.remove(&game_id);
        }
    }

    pub fn past_games(&self) -> Vec<(u64, GameStats)> {
        self.db
            .archive
            .iter()
            .flat_map(|row| {
                let (key, value) = row.ok()?;
                let key = u64::from_be_bytes(<[u8; 8]>::try_from(&*key).ok()?);
                let value = serde_json::from_slice(&value).ok()?;
                Some((key, value))
            })
            .collect()
    }

    fn random_id() -> String {
        let mut rng = rand::thread_rng();
        (0..4).map(|_| rng.gen_range('A'..='Z')).collect()
    }
}

impl Session {
    fn new(id: String, dbs: Database, options: GameOptions) -> Result<Self, GameError> {
        let game = Game::Lobby {
            options,
            players: vec![],
            min_players: options.min_players().ok_or(GameError::InvalidGameOptions)?,
            max_players: options.max_players().ok_or(GameError::InvalidGameOptions)?,
        };
        Ok(Self::hydrate(id, dbs, game))
    }

    fn hydrate(id: String, db: Database, game: Game) -> Self {
        let mut player_states = vec![];
        for _ in 0..game.num_players() {
            player_states.push(watch::channel(Value::Null).0);
        }
        Self {
            id,
            game,
            updates: watch::channel(GameUpdate::default()).0,
            db,
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
            Game::Lobby { players, max_players, .. } => {
                if players.iter().any(|n| *n == name) {
                    return Ok(());
                }
                if players.len() == *max_players {
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

        self.try_archive();
        let opts = self.game.options();
        let names = self.game.player_names();
        let seed = rand::thread_rng().next_u64();
        self.game = Game::Playing {
            game: GameInner::new(opts, &names, seed)?,
            started_ts: chrono::offset::Utc::now(),
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
        self.try_archive();

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

        self.try_archive();
        self.game = Game::GameOver;
        self.notify();
        self.persist_game().ok();

        Ok(())
    }

    /// Notifies all connected clients of the new game state.
    fn notify(&mut self) {
        let state = match &self.game {
            Game::Lobby { players, options, .. } => Self::lobby_update(players, options),
            Game::Playing { game, .. } => Self::game_update(game),
            Game::GameOver => Self::game_over_update(),
        };
        self.updates.send_replace(state);
        self.last_ts = Instant::now();
    }

    /// Creates a lobby game update.
    fn lobby_update(players: &[String], opts: &GameOptions) -> GameUpdate {
        let make_player = |name: &String| PublicPlayer {
            name: name.clone(),
            alive: true,
            not_hitler: false,
        };
        let can_start = players.len() >= opts.min_players().unwrap_or(999);
        GameUpdate {
            lifecycle: GameLifecycle::Lobby { can_start },
            players: players.iter().map(make_player).collect(),
            board_update: None,
            player_updates: vec![],
        }
    }

    /// Create a game update.
    fn game_update(game: &GameInner) -> GameUpdate {
        GameUpdate {
            lifecycle: GameLifecycle::Playing,
            players: game.get_public_players(),
            board_update: Some(game.get_board_update()),
            player_updates: (0..game.num_players()).map(|i| game.get_player_update(i)).collect(),
        }
    }

    /// Creates a game over update.
    fn game_over_update() -> GameUpdate {
        GameUpdate {
            lifecycle: GameLifecycle::Ended,
            players: vec![],
            board_update: None,
            player_updates: vec![],
        }
    }

    /// Persists the game state to disk, so it can be recovered upon server restart.
    fn persist_game(&mut self) -> Result<(), Box<dyn Error>> {
        self.db
            .game
            .insert(self.id.as_bytes(), serde_json::to_string(&self.game)?.as_bytes())?;
        Ok(())
    }

    /// Archives the game if it is over and hasn't been archived yet.
    fn try_archive(&mut self) {
        self.archive().unwrap_or_else(|err| {
            log::error!("Cannot archive game: {}: {}", &self.id, err);
        })
    }

    /// Archives the game if it is over and hasn't been archived yet.
    fn archive(&mut self) -> Result<(), Box<dyn Error>> {
        let Game::Playing { ref game, started_ts, archived } = self.game else {
            return Ok(());
        };
        if archived {
            return Ok(());
        }
        let Some(outcome) = game.outcome() else {
            return Ok(());
        };

        let stats = serde_json::to_string(&GameStats {
            id: self.id.clone(),
            started: started_ts,
            finished: chrono::offset::Utc::now(),
            players: game.player_names().map(str::to_string).collect(),
            outcome,
        })?;
        let value = Some(stats.as_bytes());

        loop {
            let key = self.next_id()?;
            match self.db.archive.compare_and_swap::<_, &[_], _>(key, None, value)? {
                Ok(_) => break,
                Err(CompareAndSwapError { .. }) => continue,
            }
        }

        if let Game::Playing { archived, .. } = &mut self.game {
            *archived = true;
        }
        Ok(())
    }

    fn next_id(&self) -> sled::Result<[u8; 8]> {
        let latest = self.db.archive.last()?.map_or(0, |(k, _)| {
            let mut bytes = [0; 8];
            bytes.copy_from_slice(&k[..k.len().min(8)]);
            u64::from_be_bytes(bytes)
        });
        Ok((latest + 1).to_be_bytes())
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

    fn options(&self) -> GameOptions {
        match self {
            Game::Lobby { options, .. } => *options,
            Game::Playing { game, .. } => game.options(),
            Game::GameOver => GameOptions::default(),
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

impl Default for GameLifecycle {
    fn default() -> Self {
        Self::Lobby { can_start: false }
    }
}
