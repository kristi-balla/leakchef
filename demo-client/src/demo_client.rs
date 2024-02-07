use anyhow::{Context, Result};
use lib_client::{LibClient, PlainIdentifierPasswordPair};
use memory_stats::memory_stats;
use std::{collections::HashMap, fs::File, io::Write};
use tokio::{task::JoinHandle, time::Interval};

pub async fn exist(
    client: LibClient,
    known_identities: HashMap<String, PlainIdentifierPasswordPair>,
    maybe_handle: Option<JoinHandle<()>>,
) -> Result<()> {
    let filter = "FOM7YjPDhpwkquBaV7gIqE+K3KDYrmk6TPrBeVKpNLA=";

    for _ in 0..1000 {
        if let Some(handle) = maybe_handle.as_ref() {
            if handle.is_finished() {
                tracing::info!("Shutdown received! Breaking out of the loop");
                break;
            }
        }

        let mut matches = 0;
        let mut total = 0;

        let (leak_id, identities) = client
            .get_latest_leak(filter, 100_000)
            .await
            .context("Something went wrong while fetching the latest metadata")?;
        total += identities.len();

        tracing::info!("start counting matched identities on the first batch");
        let part_matches = client.count_matches(known_identities.clone(), identities);
        matches += part_matches.len();

        if leak_id.is_empty() {
            tracing::warn!("leak_id is empty, we shall now return from where we came");
            return Ok(());
        }

        loop {
            let more_identities = client
                .get_leak(filter, 100_000, &leak_id)
                .await
                .context("Something went horribly wrong while fetching leaks")?;

            if more_identities.is_empty() {
                tracing::info!("This leak seems to be fully received");
                break;
            }

            total += more_identities.len();

            tracing::debug!("start counting matched identities on subsequent batch");
            let part_matches = client.count_matches(known_identities.clone(), more_identities);
            matches += part_matches.len();
        }

        client
            .send_result(&leak_id, matches as u32, total as u32)
            .await
            .context("Could not send the result")?;
    }
    Ok(())
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
