use std::str::FromStr;

use bson::{doc, oid::ObjectId};
use server::{Customer, DBClient, DatabaseError, LeakStatus, MyCache};

pub async fn setup() -> DBClient {
    // USE the test-prod for testing purposes

    DBClient::new("mongodb://localhost:27017/test", "test")
        .await
        .unwrap()
}

#[tokio::test]
pub async fn get_next_identities_unknown_leak_id() {
    let leak_id = "thisOneDoesNotExit".to_string();
    let db_client = setup().await;
    let cache = MyCache::default();
    let customer_id = 1;

    let identities = db_client
        .get_next_identities(leak_id, "".to_string(), 100, &cache, customer_id)
        .await
        .unwrap();

    assert!(identities.is_empty());
}

#[tokio::test]
pub async fn get_next_identities_known_leak_id() {
    let leak_id = "10efd8f20a6a3ea30706e5caf0566436d412f57c1c01ae3e61e4cd26738f7938".to_string();
    let db_client = setup().await;
    let cache = MyCache::default();
    let customer_id = 1;

    let actual = db_client
        .get_next_identities(
            leak_id,
            "GwWx7q5UmRISJ1S225RpQBcceM5dTYrQII50IVolS+g=".to_string(),
            100,
            &cache,
            customer_id,
        )
        .await
        .unwrap();

    assert!(actual.len() <= 100_000);
}

#[tokio::test]
pub async fn get_customer_id_wrong_api_key() {
    let api_key = "thisOneDoesNotExit".to_string();
    let db_client = setup().await;

    let result = db_client.get_customer_id(api_key).await;
    assert_eq!(result.unwrap_err(), DatabaseError::ResultIsEmpty);
}

#[tokio::test]
pub async fn get_customer_id_success() {
    let api_key = "0fb8f7bb-38aa-43e3-b6ec-7dac498aab27".to_string();
    let db_client = setup().await;

    let expected = 2341;
    let actual = db_client.get_customer_id(api_key).await.unwrap();

    assert_eq!(actual, expected);
}

#[tokio::test]
pub async fn get_last_identity_for_customer_success() {
    let customer_id = 1;
    let leak_id = "somethingLeaky==".to_string();
    let db_client = setup().await;

    db_client
        .create_status(1, "somethingLeaky==", 100)
        .await
        .unwrap();

    let actual = db_client
        .get_last_identity_for_customer(customer_id, leak_id)
        .await
        .unwrap();

    let _ = db_client.delete_status(1).await;

    assert_eq!(actual, None);
}

#[tokio::test]
pub async fn get_latest_metadata_success() {
    let customer_id = 2341;
    let db_client = setup().await;

    let expected_oid = ObjectId::parse_str("649c541268eea91ac15cce08").unwrap();
    let expected_leak_id =
        "10efd8f20a6a3ea30706e5caf0566436d412f57c1c01ae3e61e4cd26738f7938".to_string();
    let expected_parsed_identities = 100000;

    let actual = db_client
        .get_latest_metadata(customer_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(expected_oid, actual.id.unwrap());
    assert_eq!(expected_leak_id, actual.leak_id);
    assert_eq!(expected_parsed_identities, actual.extracted_identities);
}

#[tokio::test]
pub async fn get_identities_left_success() {
    let customer_id = 1;
    let leak_id = "10efd8f20a6a3ea30706e5caf0566436d412f57c1c01ae3e61e4cd26738f7938";
    let db_client = setup().await;

    db_client
        .create_status(
            1,
            "10efd8f20a6a3ea30706e5caf0566436d412f57c1c01ae3e61e4cd26738f7938",
            100,
        )
        .await
        .unwrap();

    let expected = 100;
    let actual = db_client
        .get_identities_left(customer_id, leak_id)
        .await
        .unwrap();

    let _ = db_client.delete_status(1).await;

    assert_eq!(actual, expected);
}

#[tokio::test]
pub async fn set_leak_done() {
    let customer_id = 1;
    let leak_id = "somethingLeaky==";
    let db_client = setup().await;

    db_client
        .create_status(1, "somethingLeaky==", 100)
        .await
        .unwrap();

    let result = db_client
        .set_leak_done(customer_id, leak_id.to_string())
        .await;

    let _ = db_client.delete_status(1).await;

    assert!(result.is_ok())
}

#[tokio::test]
pub async fn update_last_sent_success() {
    let customer_id = 1;
    let leak_id = "10efd8f20a6a3ea30706e5caf0566436d412f57c1c01ae3e61e4cd26738f7938";
    let last_sent_id = ObjectId::parse_str("648b7516acc3f92c04dead44").unwrap();
    let identities_read = 100;
    let leak_status = LeakStatus::Finished;
    let db_client = setup().await;

    db_client
        .create_status(
            1,
            "10efd8f20a6a3ea30706e5caf0566436d412f57c1c01ae3e61e4cd26738f7938",
            100,
        )
        .await
        .unwrap();

    let result = db_client
        .update_last_sent(
            customer_id,
            leak_id,
            Some(last_sent_id),
            identities_read,
            leak_status,
        )
        .await;

    println!("Here res: {result:?}");

    let _ = db_client.delete_status(1).await;

    assert!(result.is_ok())
}

#[tokio::test]
pub async fn update_result_success() {
    let customer_id = 1;
    let leak_id = "somethingLeaky==";
    let received_identities = 100;
    let number_of_matches = 90;
    let db_client = setup().await;

    db_client
        .create_status(1, "somethingLeaky==", 100)
        .await
        .unwrap();

    let result = db_client
        .update_result(leak_id, customer_id, received_identities, number_of_matches)
        .await;

    let _ = db_client.delete_status(1).await;

    assert!(result.is_ok());
}

#[tokio::test]
pub async fn update_handled_leaks_success() {
    let customer_id = 1;
    let leak_id = "de5f7839a1b44fcc7c023799b3367c1b";
    let db_client = setup().await;

    let customer = create_test_customer();
    let _ = insert_customer(db_client.clone(), customer.clone()).await;

    let result = db_client.update_handled_leaks(customer_id, leak_id).await;

    let _ = remove_customer(db_client, customer.clone()).await;

    assert!(result.is_ok())
}

fn create_test_customer() -> Customer {
    Customer {
        customer_id: 1,
        handled_leaks: vec![],
        customer_salt: String::from_str("cafebabe").unwrap(),
    }
}

async fn insert_customer(db_client: DBClient, customer: Customer) {
    db_client
        .customers
        .insert_one(customer, None)
        .await
        .unwrap();
}

async fn remove_customer(db_client: DBClient, customer: Customer) {
    let query = doc! {
        "customer_id": customer.customer_id
    };

    let _ = db_client.customers.delete_one(query, None).await.unwrap();
}

#[tokio::test]
pub async fn get_customer_salt_success() {
    let customer_id = 2341;
    let db_client = setup().await;

    let expected = "i55B613";
    let actual = db_client.get_customer_salt(customer_id).await.unwrap();

    assert_eq!(expected, actual);
}
