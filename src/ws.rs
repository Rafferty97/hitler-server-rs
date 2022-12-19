use crate::session::{Client, SessionManager};
use futures_util::{select, FutureExt, Sink, SinkExt, StreamExt, TryStreamExt};
use serde_json::{json, Value};
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

    let mut client = Client::new();

    loop {
        select! {
            msg = read.try_next() => match msg {
                Ok(Some(Message::Text(msg))) => {
                    process_message(msg, &mut client, manager, &mut write).await;
                    // let players = ["ALEX", "BOB", "CHARLIE", "DAVID", "ED"].map(|s| s.into());
                    // let mut game = Game::new(&players, 0);
                    // game.player_next("ALEX");
                    // game.player_next("BOB");
                    // game.player_next("CHARLIE");
                    // game.player_next("DAVID");
                    // game.player_next("ED");
                    // // let json = serde_json::to_value(&game).unwrap();
                    // write.send(Message::text(json!({
                    //     "type": "game_joined",
                    //     "gameId": "AAAA"
                    // }).to_string())).await;
                    // tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    // write.send(Message::text(json!({
                    //     "type": "update",
                    //     "state": game.get_board_json()
                    // }).to_string())).await;
                }
                _ => {
                    log::info!("Connection lost");
                    break;
                }
            },
            state = client.next_state().fuse() => {
                let msg = json!({
                    "type": "update",
                    "state": state
                }).to_string();
                if write.send(Message::text(msg)).await.is_err() {
                    log::error!("Could not send websockets message");
                    break;
                }
            }
        }
    }
}

async fn process_message(
    msg: String,
    client: &mut Client,
    manager: &SessionManager,
    write: &mut (impl Sink<Message> + Unpin),
) {
    let Ok(json) = serde_json::from_str::<Value>(&msg) else {
        log::error!("Invalid JSON received: {}", &msg);
        return;
    };

    match json["type"].as_str().unwrap_or("") {
        "create_game" => {
            let session = manager.create_game();
            let id = session.lock().unwrap().id().to_owned();
            write
                .send(Message::Text(
                    json!({
                        "type": "game_created",
                        "gameId": id
                    })
                    .to_string(),
                ))
                .await
                .ok(); // FIXME: Errors?
            log::info!("Created game: {}", id);
        }
        "board_join" => {
            let Some(id) = json["gameId"].as_str() else {
                return; // FIXME: Errors?
            };
            let Some(session) = manager.find_game(id) else {
                return; // FIXME: Errors?
            };
            client.join_as_board(session);
            write
                .send(Message::text(
                    json!({
                        "type": "game_joined",
                        "gameId": id
                    })
                    .to_string(),
                ))
                .await
                .ok(); // FIXME: Errors?
            log::info!("Board has joined game: {}", id);
        }
        "board_next" => {
            client.board_next().ok(); // FIXME: Errors?
        }
        "player_join" => {
            let Some(id) = json["gameId"].as_str() else {
                return; // FIXME: Errors?
            };
            let Some(name) = json["name"].as_str().or_else(|| json["playerId"].as_str()) else {
                return; // FIXME: Errors?
            };
            let Some(session) = manager.find_game(id) else {
                return; // FIXME: Errors?
            };
            client.join_as_player(session, name).ok(); // FIXME: Errors?
            write
                .send(Message::text(
                    json!({
                        "type": "game_joined",
                        "gameId": id,
                        "playerId": name
                    })
                    .to_string(),
                ))
                .await
                .ok(); // FIXME: Errors?
            log::info!("Player {} has joined game: {}", name, id);
        }
        "player_action" => {
            client.start_game().ok(); // FIXME
        }
        "get_state" => {
            log::info!("Unimplemented"); // FIXME
        }
        other => {
            log::error!("Unknown command type: {}", other);
        }
    }
}
