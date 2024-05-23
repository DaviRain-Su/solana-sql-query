use solana_query_service::configuration::get_configuration;
use solana_query_service::startup::run;
use sqlx::{Connection, PgConnection};
use std::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    // Panic if we can't read configuration
    let configuration = get_configuration()?;
    tracing::info!("config: {:?}", configuration);
    let connection = PgConnection::connect(&configuration.database.connection_string()).await?;
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await?;

    Ok(())
}
