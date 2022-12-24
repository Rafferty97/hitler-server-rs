use crate::{error::GameError, game::Game};
use dashmap::{mapref::entry::Entry, DashMap};
use rand::{Rng, RngCore};
use serde_json::Value;
use sled::IVec;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::watch;

pub struct SessionManager {
    sessions: DashMap<String, SessionHandle>,
    db: sled::Tree,
}

pub type SessionHandle = Arc<Mutex<Session>>;

impl SessionManager {
    pub fn new(db: sled::Tree) -> Self {
        let sessions = DashMap::new();
        let load = |entry: Result<(IVec, IVec), _>| {
            let (id, game) = entry?;
            let id = String::from_utf8(id.to_vec())?;
            let game = serde_json::from_slice(&game)?;
            let session = Session::new(id.clone(), db.clone(), game);
            let session = Arc::new(Mutex::new(session));
            sessions.insert(id, session);
            Ok(())
        };
        for entry in db.iter() {
            load(entry).unwrap_or_else(|err: Box<dyn Error>| {
                log::error!("Error loading game: {:?}", err);
            });
        }
        Self { sessions, db }
    }

    pub fn create_game(&self) -> SessionHandle {
        loop {
            let id = Self::random_id();
            let entry = self.sessions.entry(id);
            if let Entry::Occupied(_) = entry {
                continue;
            }
            let session = Session::new(entry.key().clone(), self.db.clone(), None);
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
        // Find expired sessions
        let mut expired = vec![];
        for session in self.sessions.iter_mut() {
            if session
                .lock()
                .map(|session| {
                    Instant::now().duration_since(session.last_ts) > Duration::from_secs(7200)
                })
                .unwrap_or(true)
            {
                log::info!("Found expired game: {}", session.key());
                expired.push(session.key().clone());
            }
        }

        // Delete them
        for game_id in expired.into_iter() {
            if self.db.remove(&game_id).is_ok() {
                self.sessions.remove(&game_id);
            }
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

pub struct Session {
    /// The game ID.
    id: String,
    /// The players in the game.
    players: Vec<Player>,
    /// The game itself once it has started.
    game: Option<Game>,
    /// Channel for sending board state updates.
    board_state: watch::Sender<Value>,
    /// The database the game should be persisted to.
    db: sled::Tree,
    /// Timestamp of the last time the game was interacted with.
    last_ts: Instant,
}

/// Represents an active Secret Hitler game.
impl Session {
    pub fn new(id: String, db: sled::Tree, game: Option<Game>) -> Self {
        let players = game
            .as_ref()
            .map(|g| {
                g.player_names()
                    .map(|name| Player::new(name.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let board_state = watch::channel(Value::Null).0;
        Self {
            id,
            players,
            game,
            board_state,
            db,
            last_ts: Instant::now(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn get_or_insert_player(&mut self, name: &str) -> Result<usize, GameError> {
        if let Some(idx) = self.players.iter().position(|player| player.name == name) {
            return Ok(idx);
        }

        if self.game.is_some() {
            return Err(GameError::CannotJoinStartedGame);
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

    pub fn join_board(&mut self) -> watch::Receiver<Value> {
        let rx = self.board_state.subscribe();
        self.notify();
        rx
    }

    pub fn join_player(&mut self, player: usize) -> watch::Receiver<Value> {
        let rx = self.players[player].player_state.subscribe();
        self.notify();
        rx
    }

    fn notify(&mut self) {
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
        self.last_ts = Instant::now();
    }

    fn persist_game(&self) -> Result<(), Box<dyn Error>> {
        self.db.insert(
            self.id.as_bytes(),
            serde_json::to_string(&self.game)?.as_bytes(),
        )?;
        Ok(())
    }

    fn player_names(&self) -> Vec<String> {
        self.players
            .iter()
            .map(|p| p.name.to_string())
            .collect::<Vec<_>>()
    }
}

struct Player {
    /// The player name.
    name: String,
    /// Channel for sending player state updates.
    player_state: watch::Sender<Value>,
}

impl Player {
    fn new(name: String) -> Self {
        Self {
            name,
            player_state: watch::channel(Value::Null).0,
        }
    }
}

/// A single game client, which could be a board or a player.
pub struct Client<'a> {
    manager: &'a SessionManager,
    session: Option<Arc<Mutex<Session>>>,
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

        let names = session
            .players
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>();
        let seed = rand::thread_rng().next_u64();
        session.game = Some(Game::new(&names, seed));
        session.notify();
        session.persist_game().ok();

        Ok(())
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
        session.persist_game().ok();

        Ok(())
    }
}
