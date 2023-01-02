use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sled::IVec;
use std::{error::Error, time::Duration};
use tokio_postgres::{types::ToSql, Client, NoTls, Statement};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameStats {
    pub id: String,
    pub players: Vec<String>,
    pub started: DateTime<Utc>,
    pub finished: DateTime<Utc>,
    pub outcome: Outcome,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Outcome {
    /// The liberals completed their policy track.
    LiberalPolicyTrack,
    /// The fascists completed their policy track.
    FascistPolicyTrack,
    /// The communists completed their policy track.
    CommunistPolicyTrack,
    /// Hitler was elected chancellor
    HitlerChancellor,
    /// Hitler was executed
    HitlerExecuted,
    /// The Capitalist was executed
    CapitalistExecuted,
}

impl ToString for Outcome {
    fn to_string(&self) -> String {
        match self {
            Outcome::LiberalPolicyTrack => "LiberalPolicyTrack",
            Outcome::FascistPolicyTrack => "FascistPolicyTrack",
            Outcome::CommunistPolicyTrack => "CommunistPolicyTrack",
            Outcome::HitlerChancellor => "HitlerChancellor",
            Outcome::HitlerExecuted => "HitlerExecuted",
            Outcome::CapitalistExecuted => "CapitalistExecuted",
        }
        .to_string()
    }
}

pub async fn sync_game_stats(db: sled::Db) {
    let client = match connect_pg().await {
        Ok(client) => client,
        Err(err) => return log::error!("Could not connect to PostgresQL: {:?}", err),
    };

    let sql = "INSERT INTO game (id, code, started, finished, players, outcome)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT DO NOTHING;";
    let Ok(insert) = client.prepare(sql).await else {
        return log::error!("Could not create prepared statement");
    };

    let Ok(db) = db.open_tree("archive") else {
        return log::error!("Could not open archive database");
    };

    log::info!("Writing game statistics to PostgresQL.");
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;

        if client.is_closed() {
            log::error!("Connected to PostgresQL closed.");
            return;
        }

        let Some(entry) = db.iter().flat_map(|e| e.ok().and_then(read_row)).next() else {
            continue;
        };

        if let Err(err) = write_row(&client, &insert, entry.0, entry.1).await {
            log::error!("Could not write row: {:?}", err);
            continue;
        }

        log::info!("Archived game {} to PostgresQL", entry.0);
        db.remove(entry.0.to_be_bytes()).ok();
    }
}

async fn connect_pg() -> Result<Client, Box<dyn Error>> {
    let host = std::env::var("PG_HOST")?;
    let user = std::env::var("PG_USER")?;
    let password = std::env::var("PG_PASSWORD")?;
    let dbname = std::env::var("PG_DBNAME")?;

    let (client, connection) = tokio_postgres::Config::new()
        .host(&host)
        .user(&user)
        .password(&password)
        .dbname(&dbname)
        .connect(NoTls)
        .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(client)
}

fn read_row(entry: (IVec, IVec)) -> Option<(i64, GameStats)> {
    let key = u64::from_be_bytes(entry.0.as_ref().try_into().ok()?) as i64;
    let game = serde_json::from_slice::<GameStats>(&entry.1).ok()?;
    Some((key, game))
}

async fn write_row(
    client: &Client,
    stmt: &Statement,
    key: i64,
    game: GameStats,
) -> Result<(), Box<dyn Error>> {
    let args: [&(dyn ToSql + Sync); 6] = [
        &key,
        &game.id.as_str(),
        &game.started,
        &game.finished,
        &game.players,
        &game.outcome.to_string(),
    ];
    client.execute(stmt, &args).await?;
    Ok(())
}
