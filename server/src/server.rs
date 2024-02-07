use crate::{routes, utils::MyCache, AuthService, DBClient};
use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web::Data,
    App, HttpServer,
};
use anyhow::{Context, Result};
use crusty::LiveFeedCrypto;
use tokio_shutdown::Shutdown;
use tracing::instrument;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Server {
    pub state: ServerState,
    pub shutdown: Shutdown,
}

impl Server {
    // What is used when you want the boi to run
    #[instrument(skip(self))]
    pub async fn run(self, bind_addr: &str) -> Result<()> {
        tracing::info!("Started creating HTTP-server");

        let data = Data::new(self.state);

        tracing::debug!("starting the HTTP server");
        let server_future = HttpServer::new(move || ServerState::into_app(data.clone()))
            .disable_signals()
            .bind(bind_addr)
            .context(format!("Failed binding to address '{bind_addr}'"))?
            .run();

        tracing::debug!("getting the handles of the shutdown channel and the server");
        let server_handle = server_future.handle();

        tokio::select! {
            _ = server_future => tracing::error!("something went wrong while running the server"),
            _ = self.shutdown.handle() => tracing::debug!("shutdown received! Proceeding gracefully!"),
        };

        server_handle.stop(true).await;

        tracing::info!("Goodbye!");
        Ok(())
    }
}

// 1. define a generic server state with the Database being the generic parameter
#[derive(TypedBuilder)]
pub struct ServerState {
    pub database: DBClient,
    pub crypto: LiveFeedCrypto,
    pub cache: MyCache,
}

// 2. give this struct some methods
impl ServerState {
    /// Returns an actix app factory with the configures auth middleware,
    /// shared state and API routes.
    #[instrument(skip_all)]
    fn into_app(
        data: Data<ServerState>,
    ) -> App<
        impl ServiceFactory<
            ServiceRequest,
            Response = ServiceResponse<impl MessageBody>,
            Config = (),
            InitError = (),
            Error = actix_web::Error,
        >,
    > {
        tracing::info!("Creating app and registering endpoints");
        App::new()
            .app_data(data)
            .wrap(AuthService)
            .service(routes::hello)
            .service(routes::leak)
            .service(routes::latest_leak)
            .service(routes::result)
    }
}
