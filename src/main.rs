use secrecy::ExposeSecret;
use solana_query_service::configuration::get_configuration;
use solana_query_service::startup::run;
use solana_query_service::telemetry::get_subscriber;
use solana_query_service::telemetry::init_subscriber;
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber)?;

    // Panic if we can't read configuration
    let configuration = get_configuration()?;
    tracing::info!("config: {:?}", configuration);
    let connection =
        PgPool::connect(configuration.database.connection_string().expose_secret()).await?;
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await?;

    Ok(())
}
