use crate::game::Game;
use tokio::sync::{mpsc, watch};

pub struct Session {
    players: Vec<Player>,
    game: Option<Game>,
    board_state: watch::Sender<GameState>,
    command_tx: mpsc::Sender<Command>,
    command_rx: mpsc::Receiver<Command>,
}

struct Player {
    /// The player name.
    name: String,
    /// Channel for sending player state updates.
    player_state: watch::Sender<GameState>,
}

pub struct Client {
    /// The player name.
    name: Option<String>,
    /// The channel used for sending commands.
    commands: mpsc::Sender<Command>,
    /// The channel for listening to the game state.
    state: watch::Receiver<GameState>,
}

#[derive(Clone)]
pub struct GameState {
    // FIXME
}

pub enum Command {
    BoardNext,
}

pub enum Error {
    TooManyPlayers,
}

impl Session {
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel(32);
        Self {
            players: vec![],
            game: None,
            board_state: watch::channel(GameState {}).0,
            command_tx,
            command_rx,
        }
    }

    pub fn make_board_client(&self) -> Client {
        Client {
            name: None,
            state: self.board_state.subscribe(),
            commands: self.command_tx.clone(),
        }
    }

    pub fn make_player_client(&mut self, name: String) -> Result<Client, Error> {
        // Find an existing player, or create a new one
        let player = self.players.iter().find(|player| player.name == name);
        let player = match player {
            Some(player) => player,
            None => {
                if self.players.len() == 10 {
                    return Err(Error::TooManyPlayers);
                }
                self.players.push(Player {
                    name: name.clone(),
                    player_state: watch::channel(GameState {}).0,
                });
                self.players.last().unwrap()
            }
        };

        Ok(Client {
            name: Some(name),
            state: player.player_state.subscribe(),
            commands: self.command_tx.clone(),
        })
    }
}

impl Client {
    pub async fn get_state(&mut self) -> GameState {
        self.state.changed().await;
        self.state.borrow().clone()
    }

    pub async fn board_next(&mut self) {
        self.commands.send(Command::BoardNext).await; // FIXME
    }
}
