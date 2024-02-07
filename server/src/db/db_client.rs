use crate::{
    utils::MyCache, Customer, DatabaseError, Identity, LeakResult, LeakStatus, Metadata,
    PartialIdentity, Status,
};
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{ClientOptions, UpdateOptions},
    Client, Collection, Cursor,
};

use tokio::time::{sleep, Duration};
use tracing::instrument;
use typed_builder::TypedBuilder;

// 3. define actual DBClient. Do not forget to include relevant fields, such as connection pools and such
#[derive(TypedBuilder, Debug, Clone)]
pub struct DBClient {
    // better use collections, as you are going to use A LOT of client.collection
    pub metadata: Collection<Metadata>,
    pub identities: Collection<Identity>,
    pub customers: Collection<Customer>,
    pub status: Collection<Status>,
}

// 3.1. define some basic functionality, such as new
impl DBClient {
    #[instrument]
    pub async fn new(uri: &str, db_name: &str) -> Result<DBClient, DatabaseError> {
        tracing::info!("Started creation of the DbClient");

        let mut retries = 0;
        let max_retries = 3;
        let sleep_duration = Duration::from_secs(3); // Sleep duration of 1 second (adjust as needed)

        // loop to retry connecting to DB if it fails
        let client_options = loop {
            match ClientOptions::parse(uri).await {
                Ok(ops) => break ops,
                Err(err) => {
                    if retries >= max_retries {
                        // Handle the maximum number of retries reached
                        // You can choose to propagate the error or handle it as needed
                        tracing::error!(
                            "Connecting to mongo failed after {} retries with Error: {:?}",
                            max_retries,
                            err
                        );
                    }

                    tracing::info!(
                        "Connection failed. Retrying in {} seconds...",
                        sleep_duration.as_secs()
                    );

                    retries += 1;

                    sleep(sleep_duration).await;
                }
            }
        };

        let client = Client::with_options(client_options).map_err(|_err| {
            tracing::error!("Connection failed");
            DatabaseError::Connection
        })?;

        let database = client.database(db_name);
        let metadata_collection = database.collection("metadata");
        let identities_collection = database.collection("identities");
        let customers_collection = database.collection("customers");
        let status_collection = database.collection("status");

        let db_client = DBClient::builder()
            .metadata(metadata_collection)
            .identities(identities_collection)
            .customers(customers_collection)
            .status(status_collection)
            .build();

        Ok(db_client)
    }

    #[instrument(skip(self, cache))]
    pub async fn get_next_identities(
        &self,
        leak_id: String,
        filter: String,
        limit: i64,
        cache: &MyCache,
        customer_id: i32,
    ) -> Result<Vec<PartialIdentity>, DatabaseError> {
        tracing::info!("Start reading identities");
        let key = format!("{}:{}", customer_id, leak_id.clone());

        tracing::debug!("checking the cache for an existing cursor");
        let maybe_cursor = cache.get(key.clone()).map_err(|err| {
            tracing::error!("couldn't get old cursor: {err:?}");
            DatabaseError::Cache
        })?;

        tracing::debug!("if there is a cursor, we will read it from the cache, else just create a new one and save that one");
        let mut cursor = if let Some(my_cursor) = maybe_cursor {
            my_cursor
        } else {
            self._get_cursor_without_cache(leak_id, filter)
                .await?
                .try_chunks(limit as usize)
        };

        tracing::debug!("reading next batch of identities");
        let maybe_identities: Option<Vec<Identity>> = cursor.try_next().await.map_err(|err| {
            tracing::error!("Failed to read the next chunk: {err:?}");
            DatabaseError::Collect
        })?;

        let identities = match maybe_identities {
            None => vec![],
            Some(ids) if ids.is_empty() => vec![],
            Some(ids) => {
                tracing::debug!("Saving the cursor in the cache");
                let _ = cache.set(key, cursor).map_err(|err| {
                    tracing::error!("couldn't set cursor: {err:?}");
                    DatabaseError::Cache
                })?;
                ids
            }
        };

        tracing::debug!("collected identities with a total {}", identities.len());

        let identities: Vec<PartialIdentity> = identities
            .iter()
            .map(|identity| PartialIdentity::from(identity.clone()))
            .collect();

        tracing::info!("Finish reading identities");

        Ok(identities)
    }

    pub async fn _get_cursor_without_cache(
        &self,
        leak_id: String,
        _filter: String, // !!REMOVE THIS WHEN BENCHMARKING!!
    ) -> Result<Cursor<Identity>, DatabaseError> {
        let query = doc! {
            "leak_id": leak_id.clone(),
            "password": { "$exists": true, "$ne": [] },
            "$or": [
                { "email": { "$exists": true, "$ne": [] } },
                { "phone": { "$exists": true, "$ne": [] } },
            ]
        };

        tracing::debug!("Here is the query: {}", query);

        // why 100_000? an overwhelming amount of leaks has that number of identities
        // thus making it easier to finish a whole leak in one go
        let cursor = self.identities.find(query, None).await.map_err(|err| {
            tracing::error!("An error ocurred while finding identities: {err:?}");
            DatabaseError::Find
        })?;

        tracing::debug!("found something!");

        Ok(cursor)
    }

    pub async fn get_customer_id(&self, api_key: String) -> Result<i32, DatabaseError> {
        tracing::info!("Building query to find customer_id for {api_key}");
        let query = doc! {"api_key": api_key.clone()};

        tracing::info!("Executing query");
        let cursor = self.customers.find(query, None).await.map_err(|err| {
            tracing::error!("Failed to find customer with the following api_key: {api_key}. The error is: {err:?}");
            DatabaseError::Find
        })?;

        let customers: Vec<Customer> = cursor.try_collect().await.map_err(|err| {
            tracing::error!("An error ocurred while collecting identities {err:?}");
            DatabaseError::Collect
        })?;

        match customers.first() {
            Some(customer) => Ok(customer.customer_id),
            None => Err(DatabaseError::ResultIsEmpty),
        }
    }

    pub async fn get_last_identity_for_customer(
        &self,
        customer_id: i32,
        leak_id: String,
    ) -> Result<Option<ObjectId>, DatabaseError> {
        // the whole point of this is to get the last identity sent to a customer
        // however, the customer can request multiple leaks simultaneously
        // and receive the data.
        // The problem with this is: the server can not know what leak the client is asking for
        // This is why you need the customer_id as well as the leak_id

        tracing::info!("Building query");
        let query = doc! {
            "customer_id": customer_id,
            "current_leak_id": leak_id
        };

        // TODO: rewrite for the love of god
        tracing::info!("Executing query");
        let data_result = &self
            .status
            .find_one(query.clone(), None)
            .await
            .map_err(|err| {
                tracing::error!("Failed to find customer with the following error: {err:?}");
                DatabaseError::Find
            })?;

        match data_result {
            Some(status) => {
                tracing::debug!("Here is the status: {status:?}");
                Ok(status.last_received_identity)
            }
            None => Ok(None),
        }
    }

    pub async fn get_latest_metadata(
        &self,
        customer_id: i32,
    ) -> Result<Option<Metadata>, DatabaseError> {
        let handled_leaks = self.get_handled_leaks_for_customer(customer_id).await?;
        let query = doc! {"status": "finished", "leak_id": {"$nin": handled_leaks}};
        let metadata = self.metadata.find_one(query, None).await.map_err(|err| {
            tracing::error!("Couldn't find object due to the following error: {err:?}");
            DatabaseError::Find
        })?;

        Ok(metadata)
    }

    pub async fn get_handled_leaks_for_customer(
        &self,
        customer_id: i32,
    ) -> Result<Vec<String>, DatabaseError> {
        tracing::debug!("fetching handled leaks for customer: {customer_id:?}");

        let query = doc! {"customer_id": customer_id};
        let handled_leaks = self
            .customers
            .find_one(query, None)
            .await
            .map_err(|err| {
                tracing::error!(
                    "Couldn't get leaks for customer due to the following error: {err:?}"
                );
                DatabaseError::Find
            })?
            .ok_or(DatabaseError::Find)?
            .handled_leaks;

        Ok(handled_leaks)
    }

    pub async fn create_status(
        &self,
        customer_id: i32,
        leak_id: &str,
        identities_left: u32,
    ) -> Result<(), DatabaseError> {
        let status = Status::builder()
            .customer_id(customer_id)
            .current_leak_id(leak_id)
            .identities_left(identities_left)
            .leak_status(LeakStatus::InProgress)
            .build();

        let res = self.status.insert_one(&status, None).await.map_err(|err| {
            tracing::error!("Failed to insert due to the following error: {err:?}");
            DatabaseError::Insert
        })?;

        tracing::debug!("{res:?}");

        Ok(())
    }

    pub async fn get_metadata(&self, leak_id: &str) -> Result<Metadata, DatabaseError> {
        tracing::info!("Building query to find metadata for {leak_id}");
        let query = doc! {"leak_id": leak_id};

        tracing::info!("Executing query");
        let cursor = self.metadata.find_one(query, None).await.map_err(|err| {
            tracing::error!("Failed to find customer with the following api_key: {leak_id}. The error is: {err:?}");
            DatabaseError::Find
        })?.ok_or(DatabaseError::ResultIsEmpty)?;

        Ok(cursor)
    }

    pub async fn update_status(
        &self,
        customer_id: i32,
        leak_id: &str,
        last_sent_id: Option<ObjectId>,
        identities_read: i32,
        leak_status: LeakStatus,
    ) -> Result<(), DatabaseError> {
        let query = doc! {
            "customer_id": customer_id,
            "current_leak_id": leak_id,
        };

        let current_identities_left = self.get_identities_left(customer_id, leak_id).await?;
        let new_identities_left = current_identities_left - identities_read;

        let update = if let Some(last_received_identity) = last_sent_id {
            doc! {
                "$set": {
                    "last_received_identity": last_received_identity,
                    "leak_status": leak_status,
                    "identities_left": new_identities_left
                },
                "$setOnInsert":{
                    "customer_id": customer_id,
                    "current_leak_id": leak_id,
                }
            }
        } else {
            doc! {
                "$set": {
                    "identities_left": new_identities_left,
                    "leak_status": leak_status,
                },
                "$setOnInsert":{
                    "customer_id": customer_id,
                    "current_leak_id": leak_id,
                }
            }
        };

        let options = UpdateOptions::builder().upsert(true).build();

        let result = self
            .status
            .update_one(query, update, Some(options))
            .await
            .map_err(|err| {
                tracing::error!("Failed to update last sent identity: {err:?}");
                DatabaseError::Update
            })?;

        tracing::debug!("Here is the result: {result:?}");

        Ok(())
    }

    pub async fn get_identities_left(
        &self,
        customer_id: i32,
        leak_id: &str,
    ) -> Result<i32, DatabaseError> {
        let query = doc! {
            "customer_id": customer_id,
            "current_leak_id": leak_id,
        };

        let parsed_identities = self.get_metadata(leak_id).await?.extracted_identities;

        match self.status.find_one(query, None).await.map_err(|err| {
            tracing::error!("Couldn't find object due to the following error: {err:?}");
            DatabaseError::Find
        })? {
            Some(status) => Ok(status.identities_left as i32),
            None => Ok(parsed_identities as i32),
        }
    }

    pub async fn set_leak_done(
        &self,
        customer_id: i32,
        leak_id: String,
    ) -> Result<(), DatabaseError> {
        let query = doc! {
            "customer_id": customer_id,
            "current_leak_id": leak_id,
        };

        let update = doc! {"$set": {"leak_status": LeakStatus::Finished}};

        let _result = self
            .status
            .update_one(query, update, None)
            .await
            .map_err(|_err| {
                tracing::error!("Failed to set leak_status to done");
                DatabaseError::Update
            })?;

        Ok(())
    }

    pub async fn update_last_sent(
        &self,
        customer_id: i32,
        leak_id: &str,
        last_sent_id: Option<ObjectId>,
        identities_read: i32,
        leak_status: LeakStatus,
    ) -> Result<(), DatabaseError> {
        let query = doc! {
            "customer_id": customer_id,
            "current_leak_id": leak_id,
        };

        let current_identities_left = self.get_identities_left(customer_id, leak_id).await?;
        let identities_left = current_identities_left - identities_read;

        let update = if let Some(last_received_identity) = last_sent_id {
            doc! {
                "$set": {
                    "last_received_identity": last_received_identity,
                    "leak_status": leak_status.to_string(),
                    "identities_left": identities_left
                }
            }
        } else {
            doc! {
                "$set": {
                    "identities_left": identities_left,
                    "leak_status": leak_status.to_string(),
                }
            }
        };

        let result = self
            .status
            .update_one(query, update, None)
            .await
            .map_err(|_err| {
                tracing::error!("Failed to update last sent identity");
                DatabaseError::Update
            })?;

        tracing::debug!("Here is the result: {result:?}");

        Ok(())
    }

    pub async fn update_result(
        &self,
        leak_id: &str,
        customer_id: i32,
        received_identities: u32,
        number_of_matches: u32,
    ) -> Result<(), DatabaseError> {
        let query = doc! {
            "customer_id": customer_id,
            "current_leak_id": leak_id,
        };

        let leak_result = LeakResult::builder()
            .full_matches(number_of_matches as i32)
            .identities_received(received_identities)
            .build();

        let update =
            doc! {"$set": {"leak_result": leak_result, "leak_status": LeakStatus::Finished}};

        let _result = self
            .status
            .update_one(query, update, None)
            .await
            .map_err(|err| {
                tracing::error!("Failed to update leak_resul {err:?}");
                DatabaseError::Update
            })?;

        Ok(())
    }

    pub async fn update_handled_leaks(
        &self,
        customer_id: i32,
        handled_leak_id: &str,
    ) -> Result<(), DatabaseError> {
        tracing::debug!("Updating handled_leaks for customer: {customer_id} with the following leak_id: {handled_leak_id}");

        let query = doc! {"customer_id": customer_id};
        let update = doc! {"$push": {"handled_leaks": handled_leak_id.to_string()}};
        let result = self
            .customers
            .update_one(query, update, None)
            .await
            .map_err(|err| {
                tracing::error!("Failed to update handled leaks: {err:?}");
                DatabaseError::Update
            })?;

        tracing::debug!("The update result yielded: {result:?}");
        Ok(())
    }

    /// This function is here for testing purposes only!!
    pub async fn delete_status(&self, customer_id: i32) {
        let query = doc! {
            "customer_id": customer_id
        };

        let result = self.status.delete_many(query, None).await.unwrap();
        println!("Here is the result of the deletion: {:?}", result);
    }

    pub async fn clear_status(&self) -> Result<(), DatabaseError> {
        // Delete all documents in the collection
        let result = self
            .status
            .delete_many(doc! {}, None)
            .await
            .map_err(|err| {
                println!("Something went wrong deleting everything: {err:?}");
                DatabaseError::Update
            })?;
        println!("Deleted {} documents", result.deleted_count);
        Ok(())
    }

    pub async fn clear_customer_handled_leaks(
        &self,
        api_key: impl Into<String>,
    ) -> Result<(), DatabaseError> {
        let query = doc! {"api_key": api_key.into()};
        let update = doc! {"$set": {"handled_leaks": []}};

        // Delete all documents in the collection
        let result = self
            .customers
            .update_one(query, update, None)
            .await
            .map_err(|err| {
                println!("Something went wrong deleting everything: {err:?}");
                DatabaseError::Update
            })?;
        println!("Modified {} documents", result.modified_count);
        Ok(())
    }

    pub async fn get_customer_salt(&self, customer_id: i32) -> Result<String, DatabaseError> {
        tracing::debug!("Entered get_customer_salt");

        let query = doc! {"customer_id": customer_id};

        let customer = self
            .customers
            .find_one(query, None)
            .await
            .map_err(|err| {
                tracing::error!("Something went wrong while getting the customer_salt: {err:?}");
                DatabaseError::Find
            })?
            .ok_or(DatabaseError::ResultIsEmpty)?;

        Ok(customer.customer_salt)
    }
}
