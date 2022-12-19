use crate::session::SessionManager;
use crate::ws::accept_connection;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::TcpListener;

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

    let manager = Box::<SessionManager>::leak(Default::default());

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream, manager));
    }
}
