use crate::client::SolanaClient;
use crate::email_client::EmailClient;
use crate::router::health_check;
use crate::router::subscribe;
use crate::router::{subscrib_address, txs};
use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(
    listener: TcpListener,
    connection: PgPool,
    email_client: EmailClient,
    solana_client: SolanaClient,
) -> anyhow::Result<Server> {
    tracing::info!(
        "Starting server listening on http://{}",
        listener.local_addr()?
    );
    let email_client = web::Data::new(email_client);
    let solana_client = web::Data::new(solana_client);
    // Wrap the connection in a smart pointer
    let connection = web::Data::new(connection);
    let server = HttpServer::new(move || {
        App::new()
            // Instead of `Logger::default`
            .wrap(TracingLogger::default())
            .wrap(
                Cors::default() // 或者更细化的配置
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .route("/health_check", web::get().to(health_check))
            // A new entry in our routing table for POST /subscriptions requests
            .route("/subscriptions", web::post().to(subscribe))
            .route("/address2tx", web::post().to(subscrib_address))
            .route("/txs", web::get().to(txs))
            .app_data(connection.clone())
            .app_data(email_client.clone())
            .app_data(solana_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
