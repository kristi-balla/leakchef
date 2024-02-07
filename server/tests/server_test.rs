use std::sync::{Arc, Mutex};

use actix_http::{
    body::MessageBody,
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Request,
};
use actix_web::{
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    test::{self, TestRequest},
    web::Data,
    App,
};

use cached::TimedSizedCache;
use crusty::LiveFeedCrypto;
use server::{
    hello, leak, result, AuthService, DBClient, Identifier, LeakRequest, MyCache, Reply, Response,
    ResultRequest, ServerState,
};

pub async fn setup() -> DBClient {
    // USE the test for testing purposes
    DBClient::new("mongodb://localhost:27017/test", "test")
        .await
        .unwrap()
}

const FILTER: &str = "pxS1XL6cdCM%2BLgfSkzgI3NsWYdi7yPFAGihJXbOllVk%3D";

pub async fn create_app() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    let db = setup().await;
    let crypto = LiveFeedCrypto::default();
    let data = Data::new(ServerState {
        database: db,
        crypto,
        cache: MyCache {
            store: Arc::new(Mutex::new(
                TimedSizedCache::with_size_and_lifespan_and_refresh(1000, 20, true),
            )),
        },
    });
    App::new()
        .app_data(data)
        .wrap(AuthService)
        .service(hello)
        .service(leak)
        .service(result)
}

pub fn get_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_static("Bearer:0fb8f7bb-38aa-43e3-b6ec-7dac498aab27"),
    );
    headers
}

#[tokio::test]
pub async fn test_hello() {
    println!("Started hello");
    let app = create_app().await;
    println!("Created app");
    let app = test::init_service(app).await;
    println!("Started app");

    let headers = get_headers();
    let mut request = TestRequest::get().uri("http://localhost:8080/hello");

    for (k, v) in headers {
        request = request.insert_header((k, v));
    }

    println!("constructed request");
    let response = request.send_request(&app).await;

    println!("received a response: {:?}", response.response());
    assert!(response.response().status().as_u16() == 200);
}

async fn craft_request(
    headers: Option<HeaderMap>,
    uri: impl Into<&str>,
    _leak_id: Option<String>,
) -> Request {
    let mut request = TestRequest::get().uri(uri.into());

    if let Some(header_info) = headers {
        for (k, v) in header_info {
            request = request.insert_header((k, v));
        }
    }

    let request_data = LeakRequest::builder()
        .filter(String::from("pxS1XL6cdCM+LgfSkzgI3NsWYdi7yPFAGihJXbOllVk="))
        .supported_identifiers(vec![Identifier::EMAIL])
        .limit(100)
        .build();

    let request = request
        .data(Data::new(ServerState {
            database: setup().await,
            crypto: LiveFeedCrypto::default(),
            cache: MyCache {
                store: Arc::new(Mutex::new(
                    TimedSizedCache::with_size_and_lifespan_and_refresh(1000, 20, true),
                )),
            },
        }))
        .set_json(request_data)
        .to_request();

    request
}

async fn craft_request_with_path_parameters(
    headers: Option<HeaderMap>,
    uri: impl Into<&str>,
) -> Request {
    let mut request = TestRequest::get().uri(uri.into());

    if let Some(header_info) = headers {
        for (k, v) in header_info {
            request = request.insert_header((k, v));
        }
    }

    request
        .data(Data::new(ServerState {
            database: setup().await,
            crypto: LiveFeedCrypto::default(),
            cache: MyCache {
                store: Arc::new(Mutex::new(
                    TimedSizedCache::with_size_and_lifespan_and_refresh(1000, 20, true),
                )),
            },
        }))
        .to_request()
}

#[tokio::test]
pub async fn test_leak() {
    println!("Started leak");
    let app = create_app().await;
    println!("Created app");
    let app = test::init_service(app).await;
    println!("Started app");

    let headers = get_headers();
    let request =
        craft_request_with_path_parameters(Some(headers), "http://localhost:8080/leak?filter=pxS1XL6cdCM%2BLgfSkzgI3NsWYdi7yPFAGihJXbOllVk%3D&supports_email=true&supports_phone=false&limit=100")
            .await;

    let response: Response = test::call_and_read_body_json(&app, request).await;
    assert_eq!(response.code, 200);
    assert_eq!(response.message, "Everything is fine");

    let reply = response.reply;
    if let Reply::NormalReply(normal_reply) = reply {
        let identities = normal_reply.identities;
        assert!(identities.len() <= 100_000);
        assert!(!identities.is_empty())
    }
}

#[tokio::test]
#[should_panic]
pub async fn test_auth_middleware() {
    println!("Started leak");
    let app = create_app().await;
    println!("Created app");
    let app = test::init_service(app).await;
    println!("Started app");

    let request = craft_request(None, "http://localhost:8080/leak", None).await;

    let _response: Response = test::call_and_read_body_json(&app, request).await;
}

#[tokio::test]
pub async fn test_reading_from_leak_id() {
    println!("Started leak");
    let app = create_app().await;
    println!("Created app");
    let app = test::init_service(app).await;
    println!("Started app");

    let headers = get_headers();

    // create status for given leak to initiate smth
    let db_client = setup().await;
    db_client
        .create_status(
            2341,
            "10efd8f20a6a3ea30706e5caf0566436d412f57c1c01ae3e61e4cd26738f7938",
            34465,
        )
        .await
        .unwrap();

    let uri = format!("http://localhost:8080/leak?filter={FILTER}&supports_email=true&supports_phone=false&leak_id=10efd8f20a6a3ea30706e5caf0566436d412f57c1c01ae3e61e4cd26738f7938&limit=100");

    let request = craft_request_with_path_parameters(Some(headers), &*uri).await;

    println!("here request: {request:?}");

    let response: Response = test::call_and_read_body_json(&app, request).await;
    println!("{:?}", response);
    assert_eq!(response.code, 200);

    let _ = db_client.delete_status(6111).await;

    let reply = response.reply;
    if let Reply::NormalReply(normal_reply) = reply {
        let identities = normal_reply.identities;
        assert!(identities.len() <= 100_000);
    }
}

async fn craft_post_request(
    headers: Option<HeaderMap>,
    uri: impl Into<&str>,
    leak_id: String,
) -> Request {
    let mut request = TestRequest::post().uri(uri.into());

    if let Some(header_info) = headers {
        for (k, v) in header_info {
            request = request.insert_header((k, v));
        }
    }

    let request_data = ResultRequest::builder()
        .leak_id(leak_id)
        .received_identities(150_000_u32)
        .number_of_matches(100_000_u32)
        .build();

    let request = request
        .data(Data::new(ServerState {
            database: setup().await,
            crypto: LiveFeedCrypto::default(),
            cache: MyCache {
                store: Arc::new(Mutex::new(
                    TimedSizedCache::with_size_and_lifespan_and_refresh(1000, 20, true),
                )),
            },
        }))
        .set_json(request_data)
        .to_request();

    request
}

#[tokio::test]
pub async fn test_send_result() {
    println!("Started leak");
    let app = create_app().await;
    println!("Created app");
    let app = test::init_service(app).await;
    println!("Started app");

    let headers = get_headers();
    println!("Constructed headers");

    // create status for given leak to initiate smth
    let db_client = setup().await;
    db_client
        .create_status(6111, "886a614916239c71c34c192bb1425459", 34465)
        .await
        .unwrap();
    println!("Created status");

    let request = craft_post_request(
        Some(headers),
        "http://localhost:8080/result",
        "886a614916239c71c34c192bb1425459".to_string(),
    )
    .await;

    println!("Crafted POST request");

    let response: Response = test::call_and_read_body_json(&app, request).await;
    println!("{:?}", response);
    assert_eq!(response.code, 200);

    let _ = db_client.delete_status(6111).await;

    let reply = response.reply;
    if let Reply::NormalReply(normal_reply) = reply {
        let identities = normal_reply.identities;
        assert!(identities.len() <= 100_000);
    }
}
