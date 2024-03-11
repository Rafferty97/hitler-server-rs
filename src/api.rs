use crate::session::SessionManager;
use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde::Serialize;
use serde_json::json;
use tokio::net::TcpListener;

pub async fn make_router(manager: &'static SessionManager) -> Router {
    Router::new()
        .route("/sessions", get(get_sessions))
        .route("/pastgames", get(get_past_games))
        .with_state(manager)
}

pub async fn listen(port: u16) -> TcpListener {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .expect("could not bind to port");
    log::info!("Listening on {}", listener.local_addr().unwrap());
    listener
}

async fn get_sessions(State(manager): State<&SessionManager>) -> Result<Json<impl Serialize>, StatusCode> {
    Ok(Json(json!({
        "num_sessions": manager.num_games()
    })))
}

async fn get_past_games(State(manager): State<&SessionManager>) -> Result<Json<impl Serialize>, StatusCode> {
    let games: Vec<_> = manager
        .past_games()
        .iter()
        .map(|(id, stats)| {
            let mut json = serde_json::to_value(stats).unwrap_or(json!({}));
            json.as_object_mut().unwrap().insert("id".into(), (*id).into());
            json
        })
        .collect();

    Ok(Json(json!({
        "games": games
    })))
}
