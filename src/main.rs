use crate::session::SessionManager;
use crate::ws::accept_connection;
use std::{
    error::Error,
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

    // Bind to the socket
    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port);
    let listener = TcpListener::bind(addr).await.unwrap_or_else(|err| {
        log::error!("Could not bind to address {:?}: {:?}", addr, err);
        std::process::exit(1)
    });
    log::info!("Listening on: {:?}", addr);

    // Create the session manager
    let manager = create_session_manager().unwrap_or_else(|err| {
        log::error!("Could not create session manager: {:?}", err);
        std::process::exit(1)
    });
    log::info!(
        "Created session manager. Loaded {} games",
        manager.num_games()
    );

    // Spin up background task to clean up old games
    tokio::spawn(async {
        loop {
            tokio::task::spawn_blocking(|| manager.purge_games());
            tokio::time::sleep(Duration::from_secs(15)).await;
        }
    });

    // Accept connections
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream, manager));
    }
}

fn create_session_manager() -> Result<&'static mut SessionManager, Box<dyn Error>> {
    let db = sled::open("data")?;
    let manager = SessionManager::new(db)?;
    Ok(Box::leak(Box::new(manager)))
}
