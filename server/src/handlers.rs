use actix_http::HttpMessage;
use actix_web::{
    web::{Data, Json, Query},
    HttpRequest,
};


use crate::{
    salt_identifier, LeakRequest, LeakStatus, NormalReply,
    PartialIdentity, ResultRequest, RoutesError, ServerState,
};

pub async fn get_leak(
    req: HttpRequest,
    info: Query<LeakRequest>,
    state: Data<ServerState>,
    leak_id: String,
) -> Result<NormalReply, RoutesError> {
    tracing::info!("/leak/leak_id got called");

    tracing::debug!("Extracting information from the request");
    let db_client = &state.database;
    let limit = info.limit;

    tracing::debug!("fetching customer_id from middleware");
    let customer_id = match req.extensions().get::<i32>() {
        Some(id) => *id,
        None => {
            tracing::error!("No customer_id can be found for your api_key");
            return Err(RoutesError::ApiKey);
        }
    };

    tracing::debug!("Fetching next batch of identities");
    let filter = &info.filter;
    let cache = &state.cache;
    let identities: Vec<PartialIdentity> = db_client
        .get_next_identities(leak_id.clone(), filter.clone(), limit, cache, customer_id)
        .await
        .map_err(|err| {
            tracing::error!(
                "Failed to get the next batch of identities with the following error: {err:?}"
            );
            RoutesError::DatabaseClient(err)
        })?;

    tracing::debug!("parsing last_received_identity");
    let last_received_identity = match identities.last() {
        Some(identity) => Some(identity.object_id),
        None => {
            tracing::debug!("No identity was lastly received, so the leak is fully downloaded");
            None
        }
    };

    tracing::debug!("Here is the parsed last received identity {last_received_identity:?}");
    let identities_read = identities.len() as i32;

    tracing::debug!("updating the status");
    db_client
        .update_status(
            customer_id,
            &leak_id,
            last_received_identity,
            identities_read,
            LeakStatus::InProgress,
        )
        .await
        .map_err(|err| {
            tracing::error!("Failed to update the status: {err:?}");
            RoutesError::DatabaseClient(err)
        })?;

    tracing::debug!("Fetching customer-specific salt");
    let salt = db_client.get_customer_salt(customer_id).await?;
    let crypto = &state.crypto;

    tracing::debug!("Preparing the identities vector");
    tracing::debug!("Salting the identifier from the identities with the respective salt");
    let reply_identities = salt_identifier(identities, &info.supported_identifiers, salt, crypto)?;

    tracing::debug!("Constructing reply");
    let normal_reply = NormalReply::builder()
        .customer_id(customer_id)
        .leak_id(leak_id.clone())
        .identities(reply_identities)
        .build();

    Ok(normal_reply)
}

/// # General Description
/// This function fetches the newest leak_id for the current customer.
/// For this, `db.get_latest_metadata()` is called.
/// Then, the resulting leak_id is placed in the list of leaks currently being handled, so that subsequent requests do not read the same id.
///
/// # Parameters
/// - req: `HttpRequest`
/// - state: `Data<ServerState>`
///
/// # Return
/// a `NormalReply`, or a `RoutesError` if something went wrong
pub async fn get_newest_leak(
    req: HttpRequest,
    info: Query<LeakRequest>,
    state: Data<ServerState>,
) -> Result<NormalReply, RoutesError> {
    tracing::info!("pure /leak was called");

    tracing::debug!("fetching customer_id from middleware");
    let customer_id = match req.extensions().get::<i32>() {
        Some(id) => *id,
        None => {
            tracing::error!("No customer_id can be found for your api_key");
            return Err(RoutesError::ApiKey);
        }
    };

    let db_client = &state.database;
    let limit = info.limit;

    let metadata = db_client
        .get_latest_metadata(customer_id)
        .await
        .map_err(|err| {
            tracing::error!(
                "Failed to retrieve the latest metadata with the following error {err:?}"
            );
            RoutesError::DatabaseClient(err)
        })?;

    let (leak_id, first_batch) = if let Some(metadata) = metadata {
        let leak_id = metadata.leak_id;

        tracing::debug!("Inserting {leak_id} in the list of delivered ids");
        db_client
            .update_handled_leaks(customer_id, &leak_id)
            .await
            .map_err(|err| {
                tracing::error!("Failed adding leak to handled leaks: {err:?}");
                RoutesError::DatabaseClient(err)
            })?;

        tracing::debug!("Fetching next batch of identities");
        let filter = &info.filter;
        let cache = &state.cache;
        let identities: Vec<PartialIdentity> = db_client
            .get_next_identities(leak_id.clone(), filter.clone(), limit, cache, customer_id)
            .await
            .map_err(|err| {
                tracing::error!(
                    "Failed to get the next batch of identities with the following error: {err:?}"
                );
                RoutesError::DatabaseClient(err)
            })?;

            (leak_id, identities)
    } else {
        (String::new(), vec![])
    };

    tracing::debug!("Fetching customer-specific salt");
    let salt = db_client.get_customer_salt(customer_id).await?;
    let crypto = &state.crypto;

    tracing::debug!("Preparing the identities vector");
    tracing::debug!("Salting the identifier from the identities with the respective salt");
    let reply_identities = salt_identifier(first_batch, &info.supported_identifiers, salt, crypto)?;

    let reply = NormalReply::builder()
        .customer_id(customer_id)
        .leak_id(leak_id)
        .identities(reply_identities)
        .build();

    Ok(reply)
}

pub async fn post_result(
    req: HttpRequest,
    info: Json<ResultRequest>,
    state: Data<ServerState>,
) -> Result<NormalReply, RoutesError> {
    tracing::info!("/result got called");
    let leak_id = &info.leak_id;
    let received_identities = info.received_identities;
    let number_of_matches = info.number_of_matches;
    let db_client = &state.database;

    tracing::debug!("fetching customer_id from middleware");
    let customer_id = match req.extensions().get::<i32>() {
        Some(id) => *id,
        None => {
            tracing::error!("No customer_id can be found for your api_key");
            return Err(RoutesError::ApiKey);
        }
    };

    tracing::debug!("updating the field in the database");
    db_client
        .update_result(leak_id, customer_id, received_identities, number_of_matches)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update the result of the leak");
            RoutesError::DatabaseClient(err)
        })?;

    let normal_reply = NormalReply::builder()
        .customer_id(customer_id)
        .leak_id(leak_id.clone())
        .identities(vec![])
        .build();

    tracing::info!("result has been noted");

    Ok(normal_reply)
}
