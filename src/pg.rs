use std::error::Error;
use tokio_postgres::NoTls;

pub async fn foo() -> Result<(), Box<dyn Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=secrethitler.live user=server password=5@+Yx6@)kFySy2Q4 dbname=secrethitler",
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let insert = client
        .prepare(
            "INSERT INTO game (id, code, started, finished, players, outcome)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT DO NOTHING;",
        )
        .await?;

    client
        .execute(
            &insert,
            &[
                &3i64,
                &"ABCD",
                &std::time::SystemTime::now(),
                &std::time::SystemTime::now(),
                &["Alex", "Corey", "Aidan", "Brodie"].as_slice(),
                &"Fake",
            ],
        )
        .await?;

    Ok(())
}
