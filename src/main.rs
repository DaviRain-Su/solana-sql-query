use solana_query_service::configuration::get_configuration;
use solana_query_service::startup::run;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    // Panic if we can't read configuration
    let configuration = get_configuration()?;
    tracing::info!("config: {:?}", configuration);
    let listener = TcpListener::bind("127.0.0.1:0")?;
    run(listener)?.await?;

    Ok(())
}
