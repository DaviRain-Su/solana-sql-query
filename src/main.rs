use secrecy::ExposeSecret;
use solana_query_service::configuration::get_configuration;
use solana_query_service::startup::run;
use solana_query_service::telemetry::get_subscriber;
use solana_query_service::telemetry::init_subscriber;
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber(
        "solana-query-service".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber)?;

    // Panic if we can't read configuration
    let configuration = get_configuration()?;
    tracing::info!("config: {:?}", configuration);
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    // No longer async, given that we don't actually try to connect!
    let connection =
        PgPool::connect_lazy(configuration.database.connection_string().expose_secret())?;
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await?;

    Ok(())
}
