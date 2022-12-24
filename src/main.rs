use crate::session::SessionManager;
use crate::ws::accept_connection;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};
use tokio::net::TcpListener;

mod error;
mod game;
mod session;
mod ws;

// FIXME: Implement TLS support

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::try_init().ok();

    let Ok(Ok(port)) = std::env::var("PORT").map(|s| s.parse::<u16>()) else {
        log::error!("port is unspecified or is invalid");
        return;
    };

    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port);
    let Ok(listener) = TcpListener::bind(addr).await else {
        log::error!("Could not bind to address: {:?}", addr);
        return;
    };
    log::info!("Listening on: {:?}", addr);

    let Ok(Ok(db)) = sled::open("data").map(|db| db.open_tree("games")) else {
        log::error!("Could not open game database");
        return;
    };

    let manager = SessionManager::new(db);
    let manager = Box::leak(Box::new(manager));

    log::info!(
        "Loaded {} active games from the database",
        manager.num_games()
    );

    // Spin up background task to clean up old games
    tokio::spawn(async {
        loop {
            tokio::task::spawn_blocking(|| manager.purge_games());
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });

    // Accept connections
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream, manager));
    }
}
