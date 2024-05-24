use solana_query_service::configuration::get_configuration;
use solana_query_service::startup::run;
use solana_query_service::telemetry::get_subscriber;
use solana_query_service::telemetry::init_subscriber;
use sqlx::postgres::PgPoolOptions;
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
    let connection_pool = PgPoolOptions::new()
        // `connect_lazy_with` instead of `connect_lazy`
        .connect_lazy_with(configuration.database.with_db());

    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await?;

    Ok(())
}
