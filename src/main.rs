use solana_query_service::configuration::get_configuration;
use solana_query_service::email_client::EmailClient;
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

    // Build an `EmailClient` using `configuration`
    let sender_email = configuration.email_client.sender()?;

    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        // Pass argument from configuration
        configuration.email_client.authorization_token,
    );

    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool, email_client)?.await?;

    Ok(())
}
