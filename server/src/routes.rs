use crate::{handlers, LeakRequest, Response, ResultRequest, ServerState};
use actix_http::StatusCode;
use actix_web::{
    get, post,
    web::{Data, Json, Path, Query},
    HttpRequest, Responder,
};

// 1. define the route functions, what they do and to what service they delegate
#[get("/hello")]
pub async fn hello() -> impl Responder {
    tracing::info!("hello endpoint got called");

    // yes, unwrap bc fuck you! I want my fun during development as well
    let body = reqwest::get("https://api.chucknorris.io/jokes/random")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    // 1.1. proceed with application specific operations
    let response = Response::create_empty_response(StatusCode::OK.as_u16(), body);

    Json(response)
}

#[get("/leak")]
pub async fn latest_leak(req: HttpRequest, info: Query<LeakRequest>, state: Data<ServerState>) -> impl Responder {
    match handlers::get_newest_leak(req, info, state).await {
        Ok(normal_reply) => {
            let message = String::new();

            tracing::info!("request processed successfully");
            let response = Response::create_response_with_identities(
                StatusCode::OK.as_u16(),
                message,
                normal_reply,
            );
            Json(response)
        }
        Err(err) => {
            tracing::error!("An unexpected error ocurred: {err:?}");
            let response = Response::create_empty_response(
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                format!("{err:?}"),
            );
            Json(response)
        }
    }
}

#[get("/leak/{leak_id}")]
pub async fn leak(
    req: HttpRequest,
    info: Query<LeakRequest>,
    state: Data<ServerState>,
    path: Path<String>,
) -> impl Responder {
    match handlers::get_leak(req, info, state, path.into_inner()).await {
        Ok(normal_reply) => {
            let message = if normal_reply.identities.is_empty() {
                "All identities for this leak have been received"
            } else {
                "Everything is fine"
            };

            tracing::info!("request processed successfully");
            let response = Response::create_response_with_identities(
                StatusCode::OK.as_u16(),
                message.to_string(),
                normal_reply,
            );
            Json(response)
        }
        Err(err) => {
            tracing::error!("An unexpected error ocurred: {err:?}");
            let response = Response::create_empty_response(
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                format!("{err:?}"),
            );
            Json(response)
        }
    }
}

#[post("/result")]
pub async fn result(
    req: HttpRequest,
    info: Json<ResultRequest>,
    state: Data<ServerState>,
) -> impl Responder {
    match handlers::post_result(req, info, state).await {
        Ok(normal_reply) => {
            tracing::info!("request processed successfully");
            let response = Response::create_response_with_identities(
                StatusCode::OK.as_u16(),
                "Everything is fine".to_string(),
                normal_reply,
            );
            Json(response)
        }
        Err(err) => {
            tracing::error!("An unexpected error ocurred: {err:?}");
            let response = Response::create_empty_response(
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                format!("{err:?}"),
            );
            Json(response)
        }
    }
}
