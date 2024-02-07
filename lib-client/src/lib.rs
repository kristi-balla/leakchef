use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client as ReqwestClient,
};
use serde::{Deserialize, Serialize};
use server::{Identifier, LeakRequest, MappedIdentity, Reply, Response, ResultRequest};
use std::{collections::HashMap, fmt::Display};
use typed_builder::TypedBuilder;

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum LibClientError {
    #[error("Failed to create the reqwest::Client")]
    Creation,

    #[error("Failed to send request")]
    Send,

    #[error("Body of the response has a weird form")]
    Body,

    #[error("No customer_id returned")]
    CustomerId,

    #[error("Can not successfully build request")]
    RequestBuilder,

    #[error("Serde decided to fail")]
    Serialization,
}

#[derive(Debug, TypedBuilder)]
pub struct LibClient {
    ip: String,
    port: String,
    reqwest_client: ReqwestClient,
}

impl LibClient {
    pub async fn new(
        api_key: String,
        ip: String,
        port: String,
    ) -> Result<LibClient, LibClientError> {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&api_key).unwrap());

        let reqwest_client = ReqwestClient::builder()
            .default_headers(headers)
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|err| {
                tracing::error!("Something went wrong with the creation of the client: {err:?}");
                LibClientError::Creation
            })?;

        let client = LibClient::builder()
            .ip(ip)
            .port(port)
            .reqwest_client(reqwest_client)
            .build();
        Ok(client)
    }

    pub async fn get_hello(&self) -> Result<(), LibClientError> {
        let url = format!("http://{}:{}/{}", self.ip, self.port, "hello");
        let response = self.reqwest_client.get(url).send().await.map_err(|err| {
            tracing::error!("Something went wrong while sending the request: {err:?}");
            LibClientError::Send
        })?;

        tracing::debug!("Here is the response: {response:?}");

        let body = response.text().await.map_err(|err| {
            tracing::error!("Something went wrong while fetching the body of the request: {err:?}");
            LibClientError::Body
        })?;
        tracing::debug!("Here is the body of the response: {body:?}");
        Ok(())
    }

    pub async fn get_latest_leak(
        &self,
        filter: &str,
        limit: i64,
    ) -> Result<(String, Vec<MappedIdentity>), LibClientError> {
        tracing::debug!("GET /leak just got called");
        let url = format!("http://{}:{}/leak?", self.ip, self.port);
        let body = LeakRequest::builder()
            .supported_identifiers(vec![Identifier::EMAIL])
            .filter(filter)
            .limit(limit)
            .build();

        let supported_identifiers: String = body
            .supported_identifiers
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",");

        let response = self
            .reqwest_client
            .get(url)
            .query(&[
                ("filter", body.filter.clone()),
                ("supported_identifiers", supported_identifiers),
                ("limit", body.limit.to_string()),
            ])
            .send()
            .await
            .map_err(|err| {
                tracing::error!("Something went wrong while sending the request: {err:?}");
                LibClientError::Send
            })?
            .json::<Response>()
            .await
            .map_err(|err| {
                tracing::error!("Something went wrong while deserializing the response: {err:?}");
                LibClientError::Body
            })?;

        let (leak_id, identities) = match response.reply {
            Reply::NormalReply(reply) => (reply.leak_id, reply.identities),
            _ => {
                tracing::error!("CustomerId not available! Check that the api-key is correct");
                return Err(LibClientError::CustomerId);
            }
        };

        tracing::info!("Currently working with the following leak_id: {leak_id}");

        Ok((leak_id, identities))
    }

    /// This function fetches all the relevant identities in the newest, not yet processed leak
    /// based on `filter`. This is the hash of the customer's domain
    pub async fn get_leak(
        &self,
        filter: &str,
        limit: i64,
        leak_id: impl Into<String>,
    ) -> Result<Vec<MappedIdentity>, LibClientError> {
        tracing::info!("Started fetching the latest leak!");
        let url = format!("http://{}:{}/leak/{}?", self.ip, self.port, leak_id.into());
        let body = LeakRequest::builder()
            .supported_identifiers(vec![Identifier::EMAIL])
            .filter(filter)
            .limit(limit)
            .build();

        tracing::debug!("Finished constructing the url and body");

        let more_identities = self.make_leak_request(&url, &body).await?;

        let num_of_ids = more_identities.len();
        tracing::debug!("Just fetched {num_of_ids} identities. Adding them to the collection");

        tracing::info!("Finished reading identities for your filter");
        Ok(more_identities)
    }

    async fn make_leak_request(
        &self,
        url: &str,
        body: &LeakRequest,
    ) -> Result<Vec<MappedIdentity>, LibClientError> {
        // why do it like this and not just provide the body, since reqwest should be able to parse that?
        // that's exactly what I thought, but as it turns out, it can't

        let supported_identifiers: String = body
            .supported_identifiers
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",");

        let response = self
            .reqwest_client
            .get(url)
            .query(&[
                ("filter", body.filter.clone()),
                ("supported_identifiers", supported_identifiers),
                ("limit", body.limit.to_string()),
            ])
            .send()
            .await
            .map_err(|err| {
                tracing::error!("Something went wrong while sending the request: {err:?}");
                LibClientError::Send
            })?
            .json::<Response>()
            .await
            .map_err(|err| {
                tracing::error!("Something went wrong while deserializing the response: {err:?}");
                LibClientError::Body
            })?;

        let identities = match response.reply {
            Reply::NormalReply(reply) => reply.identities,
            _ => {
                tracing::error!("CustomerId not available! Check that the api-key is correct");
                return Err(LibClientError::CustomerId);
            }
        };

        Ok(identities)
    }

    pub async fn send_result(
        &self,
        leak_id: &str,
        number_of_matches: u32,
        received_identities: u32,
    ) -> Result<(), LibClientError> {
        tracing::info!("Started sending result!");
        let url = format!("http://{}:{}/{}", self.ip, self.port, "result");
        let body = ResultRequest::builder()
            .leak_id(leak_id.to_string())
            .number_of_matches(number_of_matches)
            .received_identities(received_identities)
            .build();

        tracing::debug!("Finished constructing body, now sending request");

        let response = self
            .reqwest_client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|err| {
                tracing::error!("Something went wrong while sending the request: {err:?}");
                LibClientError::Send
            })?
            .json::<Response>()
            .await
            .map_err(|err| {
                tracing::error!("Something went wrong while deserializing the response: {err:?}");
                LibClientError::Body
            })?;

        tracing::info!("Here is the response: {response:?}");

        Ok(())
    }

    pub fn count_matches(
        &self,
        known_identities: HashMap<String, PlainIdentifierPasswordPair>,
        received_identities: Vec<MappedIdentity>,
    ) -> Vec<PlainIdentifierPasswordPair> {
        let mut plains: Vec<PlainIdentifierPasswordPair> = vec![];

        tracing::info!(
            "Starting to count the matches between the known identities and the unknown ones"
        );

        for identity in received_identities {
            let matches: Vec<PlainIdentifierPasswordPair> = identity
                .credentials
                .iter()
                .map(|value| {
                    let creds = HashedAndSaltedCredentials {
                        id_hash: value.id.clone(),
                        dt_enc: value.password.clone(),
                    }
                    .to_string();
                    known_identities.get(&creds)
                })
                .filter(|maybe_value| maybe_value.is_some())
                .map(|safe_value| safe_value.unwrap().clone()) // live with it, I checked whether there is smth there before so if you remove it, it's on you
                .collect();

            plains.extend(matches);
        }

        tracing::debug!("after the intensive counting, we got these matches: {plains:?}");
        plains
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct HashedAndSaltedCredentials {
    pub id_hash: String,
    pub dt_enc: String,
}

impl Display for HashedAndSaltedCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.id_hash, self.dt_enc)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct PlainIdentifierPasswordPair {
    pub identifier: String,
    pub password: String,
}
