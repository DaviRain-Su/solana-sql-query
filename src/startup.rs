use crate::email_client::EmailClient;
use crate::router::health_check;
use crate::router::subscribe;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(
    listener: TcpListener,
    connection: PgPool,
    email_client: EmailClient,
) -> anyhow::Result<Server> {
    tracing::info!(
        "Starting server listening on http://{}",
        listener.local_addr()?
    );
    let email_client = web::Data::new(email_client);
    // Wrap the connection in a smart pointer
    let connection = web::Data::new(connection);
    let server = HttpServer::new(move || {
        App::new()
            // Instead of `Logger::default`
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            // A new entry in our routing table for POST /subscriptions requests
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(connection.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
