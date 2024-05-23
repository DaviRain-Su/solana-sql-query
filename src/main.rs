use solana_query_service::run;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let listener = TcpListener::bind("127.0.0.1:0")?;
    run(listener)?.await?;

    Ok(())
}
