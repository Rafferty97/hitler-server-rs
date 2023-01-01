use crate::{
    client::{BoardAction, Client, PlayerAction},
    error::GameError,
    game::GameOptions,
    session::SessionManager,
};
use futures_util::{select, FutureExt, SinkExt, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

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
                let Ok(msg) = serde_json::from_str::<WsRequest>(&msg) else {
                    log::error!("Cannot parse message: {}", &msg);
                    break;
                };
                match process_request(msg, &mut client) {
                    Ok(()) => {},
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
                let msg = json!({
                    "type": "update",
                    "state": state
                });
                if write.send(Message::Text(msg.to_string())).await.is_err() {
                    log::error!("Could not send websockets message");
                    break;
                }
            }
        }
    }
}

/// A message sent by a game client to the server.
#[derive(Serialize, Deserialize)]
enum WsRequest {
    CreateGame { options: GameOptions },
    JoinAsBoard { game_id: String },
    JoinAsPlayer { game_id: String, name: String },
    LeaveGame,
    StartGame,
    BoardAction(BoardAction),
    PlayerAction(PlayerAction),
    Heartbeat,
    EndGame,
}

/// Processes a request from the client.
fn process_request(req: WsRequest, client: &mut Client) -> Result<(), GameError> {
    match req {
        WsRequest::CreateGame { options } => {
            let game_id = client.create_game(options)?;
            client.join_as_board(&game_id)?;
        }
        WsRequest::JoinAsBoard { game_id } => {
            client.join_as_board(&game_id)?;
        }
        WsRequest::JoinAsPlayer { game_id, name } => {
            client.join_as_player(&game_id, &name)?;
        }
        WsRequest::LeaveGame => client.leave(),
        WsRequest::StartGame => client.start_game()?,
        WsRequest::BoardAction(action) => {
            // Explicitely ignore errors as they will occur when there is more than one game board.
            client.board_action(action).ok();
        }
        WsRequest::PlayerAction(action) => client.player_action(action)?,
        WsRequest::EndGame => client.end_game()?,
        WsRequest::Heartbeat => client.heartbeat(),
    }
    Ok(())
}
