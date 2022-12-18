use crate::{
    game::Game,
    session::{Client, GameState},
};
use futures_util::{select, FutureExt, SinkExt, StreamExt, TryStreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

pub async fn accept_connection(stream: TcpStream) {
    log::info!("Accepted new connection");

    let Ok(stream) = tokio_tungstenite::accept_async(stream).await else {
        log::error!("Error occured during websocket handshake");
        return;
    };
    let (mut write, read) = stream.split();
    let mut read = read.fuse();

    let mut client: Option<Client> = None;

    loop {
        select! {
            msg = read.try_next() => match msg {
                Ok(Some(Message::Text(msg))) => {
                    process_message(msg, &mut client).await;
                }
                _ => {
                    log::info!("Connection lost");
                    break;
                }
            },
            state = get_state(&mut client).fuse() => {
                if write.send(Message::text(format_state(&state))).await.is_err() {
                    log::error!("Could not send websockets message");
                    break;
                }
            }
        }
    }
}

async fn process_message(msg: String, client: &mut Option<Client>) {
    let Ok(json) = serde_json::from_str::<Value>(&msg) else {
        log::error!("Invalid JSON received: {}", &msg);
        return;
    };

    match json["type"].as_str().unwrap_or("") {
        "create_game" => {
            log::info!("Creating game"); // FIXME
        }
        "board_join" => {
            log::info!("Unimplemented"); // FIXME
        }
        "board_next" => {
            if let Some(client) = client {
                client.board_next().await;
            }
        }
        "player_join" => {
            log::info!("Unimplemented"); // FIXME
        }
        "player_action" => {
            log::info!("Unimplemented"); // FIXME
        }
        "get_state" => {
            log::info!("Unimplemented"); // FIXME
        }
        other => {
            log::error!("Unknown command type: {}", other);
        }
    }
}

fn format_state(state: &GameState) -> String {
    // TODO
    "{}".to_string()
}

async fn get_state(client: &mut Option<Client>) -> GameState {
    match client {
        Some(client) => client.get_state().await,
        None => std::future::pending().await,
    }
}

async fn make_board_client(game: String) {}

async fn make_player_client(game: String, name: String) {}
