use anyhow::{Context, Result};
use crusty::LiveFeedCrypto;
use memory_stats::memory_stats;
use server::{DBClient, MyCache, Server, ServerState};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    str::FromStr,
    time::Duration,
};
use tokio::time::{interval, Interval};
use tokio_shutdown::Shutdown;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. load the environment variables
    dotenvy::dotenv().ok();

    // 2. load the logger-boi
    let log_level = dotenvy::var("LOG_LEVEL").unwrap_or("TRACE".to_string());
    let log_level = tracing::Level::from_str(&log_level).unwrap_or(Level::TRACE);
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_line_number(true)
        .with_thread_names(true)
        .pretty()
        .init();

    // 2.1. configure logger boi to include ip and port
    let ip = dotenvy::var("SERVER_IP").unwrap_or("0.0.0.0".to_string());
    let port = dotenvy::var("SERVER_PORT").unwrap_or("9999".to_string());
    tracing::info!("Hello world from tracing");

    // 3. create your db_client and other clients your application might need
    let db_url = dotenvy::var("MONGO_URL")
        .context("No MONGO_URL provided! In this case, we can not proceed")?;
    let db_client = DBClient::new(&db_url, "leaks")
        .await
        .context("Something went wrong creating the db-client")?;

    let crypto = LiveFeedCrypto::default();

    let cache = MyCache::default();

    let state = ServerState::builder()
        .database(db_client)
        .crypto(crypto)
        .cache(cache)
        .build();

    let shutdown = Shutdown::new()?;

    let bind_addr = &format!("{ip}:{port}");

    tracing::info!("Started recording memory");
    let interval = interval(Duration::from_secs(1));
    let memory_watcher = OpenOptions::new()
        .append(true)
        .create(true)
        .open("server_watcher.dat")
        .unwrap();
    tokio::spawn(record_memory_consumption(memory_watcher, interval));

    Server::builder()
        .state(state)
        .shutdown(shutdown)
        .build()
        .run(bind_addr)
        .await
}

pub async fn record_memory_consumption(mut file: File, mut interval: Interval) -> Result<()> {
    tracing::info!("Started recording memory consumption");
    loop {
        interval.tick().await;
        if let Some(usage) = memory_stats() {
            // you are interested in the resident set size, as that is how much your process is currently occupying in the RAM
            let rss = usage.physical_mem;
            writeln!(file, "{rss}")?;
        }
    }
}
