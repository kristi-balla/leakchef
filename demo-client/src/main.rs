use anyhow::Result;
use demo_client::{exist, record_memory_consumption};
use lib_client::{LibClient, PlainIdentifierPasswordPair};
use std::{
    collections::HashMap,
    env,
    fs::{File, OpenOptions},
    io::BufReader,
    path::Path,
    time::Duration,
};
use tokio::time::interval;
use tokio_shutdown::Shutdown;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_line_number(true)
        .with_thread_names(true)
        .pretty()
        .init();

    dotenvy::dotenv().ok();
    let ip = dotenvy::var("CLIENT_IP").unwrap_or("0.0.0.0".to_string());
    let port = dotenvy::var("CLIENT_PORT").unwrap_or("9999".to_string());

    tracing::debug!("start reading identities from identities.json");
    let file = File::open(Path::new("identities.json")).unwrap();
    let reader = BufReader::new(file);
    let known_identities: HashMap<String, PlainIdentifierPasswordPair> =
        serde_json::from_reader(reader).unwrap();

    let args = env::args().collect::<Vec<String>>();
    let api_key = args.get(1).unwrap();
    let auth_key = format!("Bearer:{api_key}");

    let client = LibClient::new(auth_key, ip, port).await?;

    let shutdown = Shutdown::new().unwrap();
    let handle = tokio::spawn(shutdown.handle());

    tracing::info!("Started recording memory");
    let interval = interval(Duration::from_secs(1));
    let memory_watcher = OpenOptions::new()
        .append(true)
        .create(true)
        .open("client_watcher.dat")
        .unwrap();
    tokio::spawn(record_memory_consumption(memory_watcher, interval));

    // let _ = client
    //     .get_hello()
    //     .await
    //     .context("Something went wrong while doing hello world");

    // result of hmac-sha256(key: EIDI, data: hotmail.com).to_base64
    // bc all generated identities are from hotmail.com

    // yea, after initiating all the stuff, it just exists XD
    exist(client, known_identities, Some(handle)).await?;
    Ok(())
}
