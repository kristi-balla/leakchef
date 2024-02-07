use std::{collections::HashMap, fs::File, io::BufReader, path::Path, sync::mpsc};

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use demo_client::exist;

use lib_client::{LibClient, PlainIdentifierPasswordPair};
use server::DBClient;
use tokio::{
    runtime::{Handle, Runtime},
    task::JoinHandle,
};
use tracing::Level;

fn setup(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_line_number(true)
        .with_thread_names(true)
        .pretty()
        .init();

    let mut group = c.benchmark_group("timing");
    group.sample_size(10);

    group.bench_function("iter_with_setup", |b| {
        b.to_async(&runtime).iter_batched(
            || {
                // some base stuff from ENV
                let db_url = String::from("mongodb://0.0.0.0:27017/leaks");
                let ip = dotenvy::var("CLIENT_IP").unwrap_or("0.0.0.0".to_string());
                let port = dotenvy::var("CLIENT_PORT").unwrap_or("9999".to_string());

                let (sender, receiver) = mpsc::channel();
                Handle::current().spawn(async move {
                    // create conn to DB
                    let db_client = DBClient::new(&db_url, "leaks").await.unwrap();

                    // clear unnecessary fields
                    let _ = db_client.clear_status().await.unwrap();
                    let _ = db_client
                        .clear_customer_handled_leaks("210252e1-3800-4fe5-9a02-cedeaf625525")
                        .await
                        .unwrap();

                    // get vars to use as inputs
                    let api_key = String::from("Bearer:93b38e3d-101a-481f-9108-407a2b91d378");
                    let client = LibClient::new(api_key, ip, port).await.unwrap();
                    let file = File::open(Path::new("identities.json")).unwrap();
                    let reader = BufReader::new(file);
                    let known_identities: HashMap<String, PlainIdentifierPasswordPair> =
                        serde_json::from_reader(reader).unwrap();
                    sender
                        .send((client, known_identities, Option::<JoinHandle<()>>::None))
                        .unwrap()
                });

                // whatever the setup returns, the routine will take as input
                receiver.recv().unwrap()
            },
            |(client, known_identities, maybe_handle)| async {
                exist(client, known_identities, maybe_handle).await
            },
            BatchSize::PerIteration,
        )
    });
}

criterion_group!(benches, setup);
criterion_main!(benches);
