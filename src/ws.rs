use crate::{
    error::GameError,
    session::{Client, SessionManager},
};
use futures_util::{select, FutureExt, SinkExt, StreamExt, TryStreamExt};
use serde_json::{json, Value};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

#[derive(Error, Debug)]
enum WsError {
    #[error("violation of the application-layer protocol")]
    ProtocolError,
    #[error("{0:?}")]
    GameError(GameError),
}

pub async fn accept_connection(stream: TcpStream, manager: &SessionManager) {
    log::info!("Accepted new connection");

    let Ok(stream) = tokio_tungstenite::accept_async(stream).await else {
        log::error!("Error occured during websocket handshake");
        return;
    };
    let (mut write, read) = stream.split();
    let mut read = read.fuse();

    let mut client = Client::new(manager);

    loop {
        select! {
            msg = read.try_next() => {
                let Ok(Some(Message::Text(msg))) = msg else {
                    break;
                };
                let Ok(msg) = serde_json::from_str::<Value>(&msg) else {
                    log::error!("Invalid JSON received: {}", &msg);
                    break;
                };
                let Ok(msg) = parse_request(&msg) else {
                    log::error!("Invalid message received: {}", &msg);
                    break;
                };
                match process_request(msg, &mut client) {
                    Ok(Some(reply)) => {
                        let reply = format_reply(reply);
                        write.send(Message::Text(reply.to_string())).await.ok();
                    },
                    Ok(None) => {},
                    Err(err) => {
                        let reply = json!({
                            "type": "error",
                            "error": err.to_string()
                        });
                        write.send(Message::Text(reply.to_string())).await.ok();
                    }
                }
            },
            state = client.next_state().fuse() => {
                let reply = json!({
                    "type": "update",
                    "state": state
                });
                if write.send(Message::Text(reply.to_string())).await.is_err() {
                    log::error!("Could not send websockets message");
                    break;
                }
            }
        }
    }
}

/// A message sent by a game client to the server.
enum Request {
    CreateGame,
    JoinAsBoard { game_id: String },
    JoinAsPlayer { game_id: String, name: String },
    BoardNext,
    PlayerAction(PlayerAction),
    GetState,
}

/// An action performed by the player.
enum PlayerAction {
    StartGame,
    ClickNext,
}

/// A message sent by the server to a game client.
enum Response {
    GameCreated {
        game_id: String,
    },
    GameJoined {
        game_id: String,
        player_id: Option<String>,
    },
    Error(WsError),
}

/// Parses a websockets message from the client.
fn parse_request(req: &Value) -> Result<Request, WsError> {
    match req["type"].as_str().unwrap_or("") {
        "create_game" => Ok(Request::CreateGame),
        "board_join" => {
            let game_id = req["gameId"]
                .as_str()
                .ok_or(WsError::ProtocolError)?
                .to_string();
            Ok(Request::JoinAsBoard { game_id })
        }
        "player_join" => {
            let game_id = req["gameId"]
                .as_str()
                .ok_or(WsError::ProtocolError)?
                .to_string();
            let name = req["name"]
                .as_str()
                .or_else(|| req["playerId"].as_str())
                .ok_or(WsError::ProtocolError)?
                .to_ascii_uppercase();
            Ok(Request::JoinAsPlayer { game_id, name })
        }
        "board_next" => Ok(Request::BoardNext),
        "player_action" => {
            let action = req["action"].as_str().unwrap_or("");
            let action = match action {
                "lobby" => PlayerAction::StartGame,
                "nightRound" => PlayerAction::ClickNext,
                _ => return Err(WsError::ProtocolError),
            };
            Ok(Request::PlayerAction(action))
        }
        "get_state" => Ok(Request::GetState),
        _ => Err(WsError::ProtocolError),
    }
}

/// Processes a request from the client.
fn process_request(req: Request, client: &mut Client) -> Result<Option<Response>, GameError> {
    match req {
        Request::CreateGame => {
            let game_id = client.create_game()?;
            return Ok(Some(Response::GameCreated { game_id }));
        }
        Request::JoinAsBoard { game_id } => {
            client.join_as_board(&game_id)?;
            return Ok(Some(Response::GameJoined {
                game_id,
                player_id: None,
            }));
        }
        Request::JoinAsPlayer { game_id, name } => {
            client.join_as_player(&game_id, &name)?;
            return Ok(Some(Response::GameJoined {
                game_id,
                player_id: Some(name),
            }));
        }
        Request::BoardNext => {
            client.board_next()?;
        }
        Request::PlayerAction(action) => match action {
            PlayerAction::StartGame => client.start_game()?,
            PlayerAction::ClickNext => client.player_next()?,
        },
        _ => {}
    }
    Ok(None)
}

/// Formats a reply to the client to be sent over websockets.
fn format_reply(res: Response) -> Value {
    match res {
        Response::GameCreated { game_id } => json!({
            "type": "game_created",
            "gameId": game_id
        }),
        Response::GameJoined { game_id, player_id } => json!({
            "type": "game_joined",
            "gameId": game_id,
            "playerId": player_id
        }),
        _ => unimplemented!(),
    }
}
