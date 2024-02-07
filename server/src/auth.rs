use crate::{utils::AuthError, DBClient, ServerState};
use actix_http::{body::EitherBody, HttpMessage};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web::Data,
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};
use uuid::Uuid;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct AuthService;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for AuthService
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        async fn authenticate(req: &ServiceRequest) -> Result<i32, AuthError> {
            // Extract the key from the header
            let header_value = req
                .headers()
                .get("Authorization")
                .ok_or(AuthError::MissingHeader)?;

            tracing::debug!("Here is the header: {header_value:?}");

            // Convert it to a valid string
            let key = header_value
                .to_str()
                .map_err(|_| AuthError::InvalidFormat)?;

            tracing::debug!("Here is the key: {key:?}");

            let bearer_token = extract_bearer_token(key).ok_or(AuthError::InvalidFormat)?;

            tracing::info!("Here is the bearer_token: {bearer_token:?}");

            let db_client = &req
                .app_data::<Data<ServerState>>()
                .ok_or(AuthError::Internal)?
                .database;

            match is_valid(bearer_token.to_string(), db_client.clone()).await {
                Ok(customer_id) => {
                    tracing::debug!("Here is the customer_id: {customer_id}");
                    Ok(customer_id)
                }
                Err(AuthError::InvalidFormat) => Err(AuthError::InvalidFormat),
                Err(_) => Err(AuthError::Internal),
            }
        }

        Box::pin(async move {
            match authenticate(&req).await {
                Ok(customer_id) => {
                    // Process the request further
                    req.extensions_mut().insert(customer_id);
                    if let Some(id) = req.extensions().get::<i32>() {
                        // Use the customer ID as needed
                        tracing::debug!(
                            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA Customer ID: {}",
                            id
                        );
                    }

                    service.call(req).await.map(|res| res.map_into_left_body())
                }
                Err(err) => {
                    // Return early with the error as status code and body
                    Ok(req.error_response(err).map_into_right_body())
                }
            }
        })
    }
}

// Helper function to extract the bearer token from the Authorization header
fn extract_bearer_token(header: &str) -> Option<&str> {
    const BEARER_PREFIX: &str = "Bearer:";
    match header.strip_prefix(BEARER_PREFIX) {
        Some(prefix) => Some(prefix.trim()),
        None => None,
    }
}

async fn is_valid(bearer_token: String, db_client: DBClient) -> Result<i32, AuthError> {
    tracing::info!("begin validity check for token");
    let token = Uuid::parse_str(&bearer_token).map_err(|_| AuthError::InvalidFormat)?;

    match db_client.get_customer_id(token.to_string()).await {
        Ok(customer_id) => Ok(customer_id),
        Err(err) => {
            tracing::error!("Something went wrong during validity check: {err:?}");
            Err(AuthError::InvalidFormat)
        }
    }
}
