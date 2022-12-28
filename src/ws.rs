use crate::{
    client::{Client, PlayerAction},
    error::GameError,
    game::GameOptions,
    session::SessionManager,
};
use futures_util::{select, FutureExt, SinkExt, StreamExt, TryStreamExt};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

/// Indicates a websockets application-level protocol error.
struct WsError;

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
                let reply = format_reply(Response::Update { state });
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
    CreateGame { options: GameOptions },
    JoinAsBoard { game_id: String },
    JoinAsPlayer { game_id: String, name: String },
    StartGame,
    BoardNext { state: String },
    PlayerAction(PlayerAction),
    Heartbeat,
    EndGame,
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
    Update {
        state: Value,
    },
}

/// Parses a websockets message from the client.
fn parse_request(req: &Value) -> Result<Request, WsError> {
    use WsError as PE;

    match req["type"].as_str().unwrap_or("") {
        "create_game" => Ok(Request::CreateGame {
            options: Default::default(), // FIXME
        }),
        "board_join" => {
            let game_id = req["gameId"].as_str().ok_or(PE)?.to_string();
            Ok(Request::JoinAsBoard { game_id })
        }
        "player_join" => {
            let game_id = req["gameId"].as_str().ok_or(PE)?.to_string();
            let name = req["name"]
                .as_str()
                .or_else(|| req["playerId"].as_str())
                .ok_or(PE)?
                .to_ascii_uppercase();
            Ok(Request::JoinAsPlayer { game_id, name })
        }
        "board_next" => Ok(Request::BoardNext {
            state: req["state"].as_str().unwrap_or("").to_string(),
        }),
        "player_action" => {
            let action = req["action"].as_str().unwrap_or("");
            let action = match action {
                "lobby" => return Ok(Request::StartGame),
                "nightRound" => PlayerAction::EndNightRound,
                "choosePlayer" => {
                    let name = req["data"].as_str().ok_or(PE)?.to_string();
                    PlayerAction::ChoosePlayer { name }
                }
                "vote" => {
                    let vote = req["data"].as_bool().ok_or(PE)?;
                    PlayerAction::CastVote { vote }
                }
                "legislative" => match req["data"]["type"].as_str() {
                    Some("discard") => PlayerAction::Discard {
                        index: req["data"]["idx"].as_u64().ok_or(PE)? as usize,
                    },
                    Some("veto") => PlayerAction::VetoAgenda,
                    _ => return Err(PE),
                },
                "nextRound" => PlayerAction::EndCardReveal,
                "policyPeak" => PlayerAction::EndExecutiveAction,
                "investigateParty" => PlayerAction::EndExecutiveAction,
                "vetoConsent" => {
                    if req["data"].as_bool().ok_or(PE)? {
                        PlayerAction::AcceptVeto
                    } else {
                        PlayerAction::RejectVeto
                    }
                }
                "gameover" => match req["data"].as_str() {
                    Some("restart") => return Ok(Request::StartGame),
                    Some("end") => return Ok(Request::EndGame),
                    _ => return Err(PE),
                },
                _ => return Err(PE),
            };
            Ok(Request::PlayerAction(action))
        }
        "heartbeat" => Ok(Request::Heartbeat),
        _ => Err(PE),
    }
}

/// Processes a request from the client.
fn process_request(req: Request, client: &mut Client) -> Result<Option<Response>, GameError> {
    match req {
        Request::CreateGame { options } => {
            let game_id = client.create_game(options)?;
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
        Request::BoardNext { state } => {
            // Explicitely ignore errors as they will occur when there is more than one game board.
            client.board_next(&state).ok();
        }
        Request::StartGame => client.start_game()?,
        Request::PlayerAction(action) => client.player_action(action)?,
        Request::EndGame => client.end_game()?,
        Request::Heartbeat => client.heartbeat(),
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
        Response::Update { state } => json!({
            "type": "update",
            "state": state
        }),
    }
}
